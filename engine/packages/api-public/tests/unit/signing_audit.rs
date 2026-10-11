use super::*;
use axum::{body::Body, http::Request, response::IntoResponse};
use tracing::instrument::WithSubscriber;
use tracing::{
	Subscriber,
	field::{Field, Visit},
	span::{Attributes, Id as SpanId, Record},
};

#[derive(Clone, Default)]
struct Captured(Arc<Mutex<String>>);
impl Visit for Captured {
	fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
		use std::fmt::Write;
		write!(self.0.lock().unwrap(), " {}={value:?}", field.name()).unwrap();
	}
}
impl Subscriber for Captured {
	fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
		true
	}
	fn new_span(&self, attrs: &Attributes<'_>) -> SpanId {
		attrs.record(&mut self.clone());
		SpanId::from_u64(1)
	}
	fn record(&self, _: &SpanId, values: &Record<'_>) {
		values.record(&mut self.clone());
	}
	fn record_follows_from(&self, _: &SpanId, _: &SpanId) {}
	fn event(&self, event: &tracing::Event<'_>) {
		self.0.lock().unwrap().push_str(event.metadata().target());
		event.record(&mut self.clone());
		self.0.lock().unwrap().push('\n');
	}
	fn enter(&self, _: &SpanId) {}
	fn exit(&self, _: &SpanId) {}
}

#[tokio::test]
async fn signing_audit_captures_safe_fields_without_payloads_tokens_or_remote_errors() {
	let captured = Captured::default();
	let dispatch = tracing::Dispatch::new(captured.clone());
	tokio::spawn(
		async move {
			let audit = Audit::default();
			audit.identity(Id::new_v1(1), Id::new_v1(1));
			audit.purpose("durable-streams.callback");
			audit.key("ds_public-key".into(), 7);
			let request = Request::builder()
				.header("content-type", "application/octet-stream")
				.header("authorization", "Bearer BEARER_SECRET_SENTINEL")
				.header("webhook-signature", "HEADER_SIGNATURE_SECRET_SENTINEL")
				.body(Body::from("BODY_SECRET_SENTINEL"))
				.unwrap();
			let mut guard = Guard::start(audit.clone(), "/auth/webhooks/sign").unwrap();
			let (body, permit) = rivet_api_peer::auth::webhook_signing::read_body_observed(
				request,
				1024,
				|_| async { Ok(()) },
				|bytes| audit.bytes(bytes),
			)
			.await
			.unwrap();
			assert_eq!(body, b"BODY_SECRET_SENTINEL");
			drop(permit);
			let response = axum::Json(serde_json::json!({"signature":"SIGNATURE_SECRET_SENTINEL"}))
				.into_response();
			guard.response(&response);
			drop(guard);

			let claims: serde_json::Value = crate::auth::strict_json::parse(
				br#"{"claims":{"ds":{"data":"CLAIMS_SECRET_SENTINEL"}}}"#,
			)
			.unwrap();
			assert!(claims["claims"].is_object());
			let mut guard = Guard::start(audit.clone(), "/auth/application-tokens").unwrap();
			guard.response(
				&axum::Json(serde_json::json!({"token":"JWT_SECRET_SENTINEL"})).into_response(),
			);
			drop(guard);

			let malformed = crate::auth::strict_json::parse::<serde_json::Value>(
				br#"{"claims":"MALFORMED_SECRET_SENTINEL","claims":1}"#,
			)
			.unwrap_err();
			for error in [
				malformed,
				crate::errors::Signing::PayloadTooLarge.build(),
				crate::errors::Signing::Unavailable.build(),
				anyhow::anyhow!("storage error: STORAGE_SECRET_SENTINEL"),
				anyhow::anyhow!("signing error: SEED_SECRET_SENTINEL"),
				anyhow::Error::new(rivet_api_builder::RawErrorResponse(
					axum::http::StatusCode::TOO_MANY_REQUESTS,
					rivet_api_builder::ErrorResponse {
						group: "signing".into(),
						code: "throttled".into(),
						message: "REMOTE_SECRET_SENTINEL".into(),
						metadata: Some(
							serde_json::json!({"secret":"REMOTE_METADATA_SECRET_SENTINEL"}),
						),
					},
				)),
			] {
				let mut guard = Guard::start(audit.clone(), "/auth/webhooks/sign").unwrap();
				guard.response(&crate::auth::signing::error_response(error));
			}
			// Dropping an unfinished request reports cancellation, using only admitted fields.
			drop(Guard::start(audit, "/auth/webhooks/sign").unwrap());
		}
		.with_subscriber(dispatch),
	)
	.await
	.unwrap();
	let output = captured.0.lock().unwrap().clone();
	assert!(
		!output.contains("SECRET_SENTINEL"),
		"secret appeared in captured tracing"
	);
	for required in [
		"rivet_signing_audit",
		"durable-streams.callback",
		"ds_public-key",
		"generation=Some(7)",
		"bytes=20",
		"signing.invalid_input",
		"signing.payload_too_large",
		"signing.unavailable",
		"signing.throttled",
		"cancelled",
		"success",
	] {
		assert!(
			output.contains(required),
			"missing safe audit field {required}: {output}"
		);
	}
}
