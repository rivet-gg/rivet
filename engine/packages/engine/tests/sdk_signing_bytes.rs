//! The generated Rust client must preserve arbitrary HTTP request bytes.
use axum::{Json, Router, body::Bytes, http::HeaderMap, routing::post};
use serde_json::json;

#[tokio::test]
async fn generated_rust_signing_client_sends_exact_bytes() {
	for payload in [
		vec![],
		vec![0],
		vec![255, 254, 128],
		b"\r\n{ \"a\": 1 }\r\n".to_vec(),
	] {
		let expected = payload.clone();
		let app = Router::new().route(
			"/auth/webhooks/sign",
			post(move |headers: HeaderMap, body: Bytes| async move {
				assert_eq!(headers["content-type"], "application/octet-stream");
				assert_eq!(headers["authorization"], "Bearer sdk-test");
				assert_eq!(&body[..], expected.as_slice());
				Json(json!({"timestamp":"123", "kid":"ds_test", "signature":"test"}))
			}),
		);
		let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
		let address = listener.local_addr().unwrap();
		let server = tokio::spawn(async { axum::serve(listener, app).await.unwrap() });
		let mut configuration = rivet_api_full::apis::configuration::Configuration::new();
		configuration.base_path = format!("http://{address}");
		configuration.bearer_access_token = Some("sdk-test".into());
		let response = rivet_api_full::apis::auth_webhook_signing_api::auth_webhooks_sign(
			&configuration,
			payload,
		)
		.await
		.unwrap();
		assert_eq!(response.timestamp, "123");
		server.abort();
	}
}
