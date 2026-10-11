//! Safe, bounded observability fields. Never retain a credential or signed input.
use axum::response::Response;
use rivet_util::Id;
use std::{
	sync::{Arc, Mutex},
	time::Instant,
};

#[derive(Clone, Default)]
pub struct Audit(Arc<Mutex<Fields>>);
#[derive(Default)]
struct Fields {
	caller: Option<Id>,
	namespace: Option<Id>,
	kid: Option<String>,
	generation: Option<u64>,
	bytes: u64,
	purpose: Option<String>,
}
impl Audit {
	pub fn purpose(&self, purpose: &str) {
		self.0.lock().unwrap().purpose = Some(purpose.to_owned());
	}
	pub fn caller(&self, caller: Id) {
		self.0.lock().unwrap().caller = Some(caller);
	}
	pub fn identity(&self, caller: Id, namespace: Id) {
		let mut f = self.0.lock().unwrap();
		f.caller = Some(caller);
		f.namespace = Some(namespace);
	}
	pub fn key(&self, kid: String, generation: u64) {
		let mut f = self.0.lock().unwrap();
		f.kid = Some(kid);
		f.generation = Some(generation);
	}
	pub fn bytes(&self, bytes: u64) {
		self.0.lock().unwrap().bytes = bytes;
	}
}
pub struct Guard {
	audit: Audit,
	operation: &'static str,
	started: Instant,
	outcome: &'static str,
	error_code: Option<String>,
}
impl Guard {
	pub fn start(audit: Audit, path: &str) -> Option<Self> {
		let operation = match path {
			"/auth/application-tokens" => "issue_token",
			"/auth/webhooks/sign" => "sign_webhook",
			"/auth/webhooks/ensure" => "ensure_webhook",
			_ if path.ends_with("/auth/webhook-keys/rotate") => "rotate_webhook",
			_ if path.ends_with("/auth/webhook-keys/revoke") => "revoke_webhook",
			_ if path.ends_with("/auth/webhook-keys") => "status_webhook",
			_ if path.ends_with("/auth/webhooks/jwks.json") => "discover_webhook",
			_ if path.ends_with("/auth/webhooks") => "metadata_webhook",
			_ => return None,
		};
		Some(Self {
			audit,
			operation,
			started: Instant::now(),
			outcome: "cancelled",
			error_code: None,
		})
	}
	pub fn response(&mut self, response: &Response) {
		self.error_code = response
			.extensions()
			.get::<rivet_api_builder::ErrorExt>()
			.map(|e| format!("{}.{}", e.group, e.code));
		self.outcome = match response.status().as_u16() {
			200..=299 => "success",
			400 | 415 | 422 => "invalid_input",
			401 => "invalid_token",
			403 => "denied",
			404 => "not_found",
			409 => "conflict",
			413 => "too_large",
			429 => "throttled",
			_ => "unavailable",
		};
	}
}
impl Drop for Guard {
	fn drop(&mut self) {
		let f = self.audit.0.lock().unwrap();
		let purpose = if self.operation == "issue_token" {
			"application"
		} else {
			"webhook"
		};
		let elapsed = self.started.elapsed().as_secs_f64();
		tracing::info!(target: "rivet_signing_audit", operation = self.operation, purpose = f.purpose.as_deref().unwrap_or(purpose), purpose_class = purpose, outcome = self.outcome, error_code = ?self.error_code,
            caller = ?f.caller, namespace = ?f.namespace, kid = ?f.kid, generation = ?f.generation,
            bytes = f.bytes, latency_seconds = elapsed, "signing request completed");
		rivet_auth_jwt::metrics::SIGNING_REQUESTS
			.with_label_values(&[self.operation, self.outcome])
			.inc();
		rivet_auth_jwt::metrics::SIGNING_LATENCY
			.with_label_values(&[self.operation, self.outcome])
			.observe(elapsed);
		rivet_auth_jwt::metrics::SIGNING_BYTES
			.with_label_values(&[self.operation])
			.inc_by(f.bytes);
	}
}
static REQUESTS: std::sync::LazyLock<Arc<tokio::sync::Semaphore>> =
	std::sync::LazyLock::new(|| Arc::new(tokio::sync::Semaphore::new(64)));
pub fn small_body_permit() -> anyhow::Result<tokio::sync::OwnedSemaphorePermit> {
	REQUESTS
		.clone()
		.try_acquire_owned()
		.map_err(|_| crate::errors::Signing::Throttled.build())
}

#[cfg(test)]
#[path = "../../tests/unit/signing_audit.rs"]
mod tests;
