use std::{future::Future, time::Duration};

use reqwest::{Client, Response, StatusCode};
use serde_json::{Value, json};

#[path = "common/api/mod.rs"]
mod api;
#[path = "common/ctx.rs"]
mod ctx;

fn test_admin_token() -> String {
	// Every test uses a fresh database; do not reuse a credential across process-wide caches.
	format!("jwt-test-admin-{}", rand::random::<u64>())
}

#[test]
fn publishes_public_only_jwks_for_issued_tokens() {
	let admin_token = test_admin_token();
	run(
		ctx::TestOpts::new(2)
			.with_auth_admin_token(admin_token.clone())
			.with_timeout(90),
		move |ctx| async move {
			let client = Client::new();
			let follower_base = format!("http://127.0.0.1:{}", ctx.get_dc(2).guard_port());
			let namespace = format!("jwt-jwks-test-{}", rand::random::<u16>());
			api::peer::namespaces_create(
				ctx.leader_dc().api_peer_port(),
				rivet_api_peer::namespaces::CreateRequest {
					name: namespace.clone(),
					display_name: "JWKS Test".into(),
				},
			)
			.await
			.expect("failed to create JWKS namespace");
			let issued = retry_success(|| {
				client
					.post(format!("{follower_base}/auth/tokens"))
					.bearer_auth(&admin_token)
					.json(&json!({
						"namespace": namespace,
						"duration": 60,
						"grants": [{"resource": "actor", "target": "any", "operations": ["read"]}]
					}))
			})
			.await
			.json::<rivet_api_types::auth::tokens::CreateResponse>()
			.await
			.expect("failed to issue JWT");
			let kid = rivet_auth_jwt::peek_header(&issued.token)
				.unwrap()
				.kid
				.to_string();
			let response = client
				.get(format!("{follower_base}/.well-known/jwks.json"))
				.send()
				.await
				.expect("failed to fetch JWKS");
			assert_eq!(response.status(), StatusCode::OK);
			assert_eq!(
				response.headers()["content-type"],
				"application/jwk-set+json"
			);
			let document: Value = response.json().await.expect("failed to decode JWKS");
			let keys = document["keys"].as_array().expect("JWKS must have keys");
			let key = keys
				.iter()
				.find(|key| key["kid"] == kid)
				.expect("signing key must be published");
			assert_eq!(key["kty"], "OKP");
			assert_eq!(key["crv"], "Ed25519");
			assert_eq!(key["alg"], "EdDSA");
			assert!(
				key.get("d").is_none(),
				"JWKS must never contain private keys"
			);
		},
	);
}

#[test]
fn registers_and_removes_namespace_bound_jwks_issuers_from_a_follower() {
	let admin_token = test_admin_token();
	run(
		ctx::TestOpts::new(2)
			.with_auth_admin_token(admin_token.clone())
			.with_timeout(90),
		move |ctx| async move {
			let namespace = format!("jwt-external-test-{}", rand::random::<u16>());
			api::peer::namespaces_create(
				ctx.leader_dc().api_peer_port(),
				rivet_api_peer::namespaces::CreateRequest {
					name: namespace.clone(),
					display_name: "External JWT Test".into(),
				},
			)
			.await
			.expect("failed to create external JWT namespace");
			let client = Client::new();
			let base = format!("http://127.0.0.1:{}", ctx.get_dc(2).guard_port());
			let endpoint = format!("{base}/namespaces/{namespace}/auth/issuers/example");
			let body = json!({
				"issuer": "https://jwt-issuer.gameinc.io",
				"audience": "rivet-api",
				"jwks_url": "https://jwt-issuer.gameinc.io/.well-known/jwks.json",
				"algorithm": "EdDSA",
				"grant_ceiling": [{
					"resource": "actor", "target": "any", "operations": ["read"]
				}]
			});
			let created = client
				.put(&endpoint)
				.bearer_auth(&admin_token)
				.json(&body)
				.send()
				.await
				.expect("failed to register JWKS issuer");
			assert_eq!(
				created.status(),
				StatusCode::OK,
				"{}",
				created.text().await.unwrap_or_default()
			);
			let manager_jwt = retry_success(|| {
				client
					.post(format!("{base}/auth/tokens"))
					.bearer_auth(&admin_token)
					.json(&json!({
						"namespace": namespace,
						"duration": 60,
						"grants": [{"resource": "actor", "target": "any", "operations": ["read"]}]
					}))
			})
			.await
			.json::<rivet_api_types::auth::tokens::CreateResponse>()
			.await
			.expect("failed to issue a valid JWT for the enrollment check");
			let attempted_enrollment = client
				.put(&endpoint)
				.bearer_auth(&manager_jwt.token)
				.json(&body)
				.send()
				.await
				.expect("failed to test JWT self-enrollment");
			assert_eq!(
				attempted_enrollment.status(),
				StatusCode::FORBIDDEN,
				"a JWT must not manage its own JWKS issuer registration"
			);
			let listed = client
				.get(format!("{base}/namespaces/{namespace}/auth/issuers"))
				.bearer_auth(&admin_token)
				.send()
				.await
				.expect("failed to list issuers");
			assert_eq!(listed.status(), StatusCode::OK);
			let issuers: Value = listed.json().await.expect("invalid issuer list");
			assert_eq!(issuers["issuers"].as_array().unwrap().len(), 1);
			assert_eq!(issuers["issuers"][0]["issuer"], body["issuer"]);
			let removed = client
				.delete(&endpoint)
				.bearer_auth(&admin_token)
				.send()
				.await
				.expect("failed to remove JWKS issuer");
			assert_eq!(removed.status(), StatusCode::OK);
			let listed = client
				.get(format!("{base}/namespaces/{namespace}/auth/issuers"))
				.bearer_auth(&admin_token)
				.send()
				.await
				.expect("failed to list issuers after removal");
			assert_eq!(listed.status(), StatusCode::OK);
			let issuers: Value = listed
				.json()
				.await
				.expect("invalid issuer list after removal");
			assert!(issuers["issuers"].as_array().unwrap().is_empty());
		},
	);
}

#[test]
fn issues_and_inspects_jwts_from_a_non_writer_datacenter() {
	let admin_token = test_admin_token();
	run(
		ctx::TestOpts::new(2)
			.with_auth_admin_token(admin_token.clone())
			.with_timeout(90),
		move |ctx| async move {
			let namespace = format!("jwt-test-{}", rand::random::<u16>());
			let created = api::peer::namespaces_create(
				ctx.leader_dc().api_peer_port(),
				rivet_api_peer::namespaces::CreateRequest {
					name: namespace.clone(),
					display_name: "JWT Test".into(),
				},
			)
			.await
			.expect("failed to create JWT test namespace");

			let client = Client::new();
			let follower_base = format!("http://127.0.0.1:{}", ctx.get_dc(2).guard_port());
			let issue_body = json!({
				"namespace": namespace,
				"duration": 60,
				"subject": "end-user-123",
				"grants": [{
					"resource": "actor",
					"target": "any",
					"operations": ["list"]
				}]
			});

			let issued = retry_success(|| {
				client
					.post(format!("{follower_base}/auth/tokens"))
					.bearer_auth(&admin_token)
					.json(&issue_body)
			})
			.await
			.json::<rivet_api_types::auth::tokens::CreateResponse>()
			.await
			.expect("failed to decode JWT issuance response");

			let inspected = client
				.get(format!("{follower_base}/auth/tokens/inspect"))
				.bearer_auth(&issued.token)
				.send()
				.await
				.expect("failed to inspect JWT");
			assert_eq!(inspected.status(), StatusCode::OK);
			let inspected = inspected
				.json::<rivet_api_types::auth::tokens::InspectResponse>()
				.await
				.expect("failed to decode JWT inspection response");
			assert_eq!(inspected.namespace_id, created.namespace.namespace_id);
			assert_eq!(inspected.subject.as_deref(), Some("end-user-123"));
			assert_eq!(inspected.grants.len(), 1);
			assert_eq!(inspected.issued_ts, issued.issued_ts);
			assert_eq!(inspected.expires_ts, issued.expires_ts);

			let actor_list = client
				.get(format!("{follower_base}/actors/names"))
				.bearer_auth(&issued.token)
				.query(&[("namespace", namespace.as_str())])
				.send()
				.await
				.expect("failed to list actors with a JWT");
			assert_eq!(actor_list.status(), StatusCode::OK);

			let acl_actor_list = client
				.get(format!("{follower_base}/actors/names"))
				.bearer_auth(&admin_token)
				.query(&[("namespace", namespace.as_str())])
				.send()
				.await
				.expect("failed to list actors with an ACL credential");
			assert_eq!(acl_actor_list.status(), StatusCode::OK);

			let missing_credential = client
				.get(format!("{follower_base}/actors/names"))
				.query(&[("namespace", namespace.as_str())])
				.send()
				.await
				.expect("failed to test a missing credential");
			assert_error_response(
				missing_credential,
				StatusCode::UNAUTHORIZED,
				"invalid_token",
			)
			.await;

			let create_acl = |body: Value| {
				client
					.post(format!("{follower_base}/acl/tokens"))
					.bearer_auth(&admin_token)
					.json(&body)
			};
			let limited_acl = create_acl(json!({ "roles": [] }))
				.send()
				.await
				.expect("failed to create ACL credential without grants");
			assert_eq!(limited_acl.status(), StatusCode::OK);
			let limited_acl = limited_acl
				.json::<rivet_api_types::acl::tokens::CreateResponse>()
				.await
				.expect("failed to decode ACL credential");
			let denied_acl = client
				.get(format!("{follower_base}/actors/names"))
				.bearer_auth(&limited_acl.token)
				.query(&[("namespace", namespace.as_str())])
				.send()
				.await
				.expect("failed to test insufficient ACL grants");
			assert_error_response(
				denied_acl,
				StatusCode::FORBIDDEN,
				"insufficient_permissions",
			)
			.await;

			let expired_acl = create_acl(json!({ "roles": [], "duration": 0 }))
				.send()
				.await
				.expect("failed to create expired ACL credential");
			assert_eq!(expired_acl.status(), StatusCode::OK);
			let expired_acl = expired_acl
				.json::<rivet_api_types::acl::tokens::CreateResponse>()
				.await
				.expect("failed to decode expired ACL credential");
			tokio::time::sleep(Duration::from_millis(5)).await;
			let expired_acl_response = client
				.get(format!("{follower_base}/actors/names"))
				.bearer_auth(&expired_acl.token)
				.query(&[("namespace", namespace.as_str())])
				.send()
				.await
				.expect("failed to test expired ACL credential");
			assert_error_response(
				expired_acl_response,
				StatusCode::UNAUTHORIZED,
				"token_expired",
			)
			.await;

			// The ACL store deliberately contains this reserved-looking bearer. It must still take
			// the JWT path and fail closed, never becoming an authorized ACL request.
			let reserved_acl_token = format!(
				"eyJ0eXAiOiJyaXZldC1hdXRoK2p3dCIsImFsZyI6IkVkRFNBIn0.invalid.{:016x}",
				rand::random::<u64>()
			);
			let reserved_acl = create_acl(json!({
				"roles": ["admin"],
				"token": reserved_acl_token,
			}))
			.send()
			.await
			.expect("failed to create reserved-looking ACL credential");
			assert_eq!(reserved_acl.status(), StatusCode::OK);
			let reserved_acl_response = client
				.get(format!("{follower_base}/actors/names"))
				.bearer_auth(&reserved_acl_token)
				.query(&[("namespace", namespace.as_str())])
				.send()
				.await
				.expect("failed to test reserved JWT fallback");
			assert_error_response(
				reserved_acl_response,
				StatusCode::UNAUTHORIZED,
				"invalid_token",
			)
			.await;

			let insufficient_body = json!({
				"namespace": namespace,
				"duration": 60,
				"grants": [{
					"resource": "actor",
					"target": "any",
					"operations": ["read"]
				}]
			});
			let insufficient = retry_success(|| {
				client
					.post(format!("{follower_base}/auth/tokens"))
					.bearer_auth(&admin_token)
					.json(&insufficient_body)
			})
			.await
			.json::<rivet_api_types::auth::tokens::CreateResponse>()
			.await
			.expect("failed to decode insufficient JWT issuance response");
			let insufficient = client
				.get(format!("{follower_base}/actors/names"))
				.bearer_auth(&insufficient.token)
				.query(&[("namespace", namespace.as_str())])
				.send()
				.await
				.expect("failed to test an insufficient JWT grant");
			let error = assert_error_response(
				insufficient,
				StatusCode::FORBIDDEN,
				"insufficient_permissions",
			)
			.await;
			assert_eq!(error["group"], "auth");

			let unknown_token = client
				.get(format!("{follower_base}/actors/names"))
				.bearer_auth("unknown-opaque-token")
				.query(&[("namespace", namespace.as_str())])
				.send()
				.await
				.expect("failed to test an unknown token");
			let error =
				assert_error_response(unknown_token, StatusCode::UNAUTHORIZED, "invalid_token")
					.await;
			assert_eq!(error["group"], "auth");

			let admin_on_jwt_endpoint = client
				.get(format!("{follower_base}/auth/tokens/inspect"))
				.bearer_auth(ADMIN_TOKEN)
				.send()
				.await
				.expect("failed to test administrator token on JWT inspection");
			let error = assert_error_response(
				admin_on_jwt_endpoint,
				StatusCode::UNAUTHORIZED,
				"invalid_token",
			)
			.await;
			assert_eq!(error["group"], "auth");

			let jwt_reissuance = client
				.post(format!("{follower_base}/auth/tokens"))
				.bearer_auth(&issued.token)
				.json(&issue_body)
				.send()
				.await
				.expect("failed to test JWT reissuance");
			let error = assert_error_response(
				jwt_reissuance,
				StatusCode::FORBIDDEN,
				"insufficient_permissions",
			)
			.await;
			assert_eq!(error["group"], "auth");

			let malformed_jwt = client
				.get(format!("{follower_base}/auth/tokens/inspect"))
				.bearer_auth("eyJ0eXAiOiJyaXZldC1hdXRoK2p3dCIsImFsZyI6IkVkRFNBIn0.invalid.invalid")
				.send()
				.await
				.expect("failed to inspect malformed JWT");
			let error =
				assert_error_response(malformed_jwt, StatusCode::UNAUTHORIZED, "invalid_token")
					.await;
			assert_eq!(error["group"], "auth");

			let old_route = client
				.post(format!(
					"{follower_base}/namespaces/{}/auth/tokens",
					created.namespace.name
				))
				.bearer_auth(&admin_token)
				.json(&issue_body)
				.send()
				.await
				.expect("failed to test removed namespace-prefixed JWT route");
			assert_eq!(old_route.status(), StatusCode::NOT_FOUND);
		},
	);
}

#[test]
fn expired_jwt_returns_auth_token_expired() {
	let admin_token = test_admin_token();
	run(
		ctx::TestOpts::new(1)
			.with_auth_admin_token(admin_token.clone())
			.with_timeout(90),
		move |ctx| async move {
			let namespace = format!("jwt-expired-test-{}", rand::random::<u16>());
			api::peer::namespaces_create(
				ctx.leader_dc().api_peer_port(),
				rivet_api_peer::namespaces::CreateRequest {
					name: namespace.clone(),
					display_name: "Expired JWT Test".into(),
				},
			)
			.await
			.expect("failed to create JWT namespace");

			let client = Client::new();
			let base = format!("http://127.0.0.1:{}", ctx.leader_dc().guard_port());
			let issued = retry_success(|| {
				client
					.post(format!("{base}/auth/tokens"))
					.bearer_auth(&admin_token)
					.json(&json!({
						"namespace": namespace,
						"duration": 1,
						"grants": [{ "resource": "actor", "target": "any", "operations": ["list"] }]
					}))
			})
			.await
			.json::<rivet_api_types::auth::tokens::CreateResponse>()
			.await
			.expect("failed to decode short-lived JWT");

			// Verification permits a small clock skew after exp; wait past the same protocol bound.
			tokio::time::sleep(Duration::from_secs(rivet_auth_jwt::PROTOCOL_CLOCK_SKEW + 2)).await;
			let expired = client
				.get(format!("{base}/actors/names"))
				.bearer_auth(&issued.token)
				.query(&[("namespace", namespace.as_str())])
				.send()
				.await
				.expect("failed to test expired JWT");
			assert_error_response(expired, StatusCode::UNAUTHORIZED, "token_expired").await;
		},
	);
}

fn run<F, Fut>(opts: ctx::TestOpts, test_fn: F)
where
	F: FnOnce(ctx::TestCtx) -> Fut,
	Fut: Future<Output = ()>,
{
	let runtime = tokio::runtime::Runtime::new().expect("failed to build runtime");
	runtime.block_on(async {
		let timeout = Duration::from_secs(opts.timeout_secs);
		let ctx = ctx::TestCtx::new_with_opts(opts)
			.await
			.expect("build testctx");
		tokio::time::timeout(timeout, test_fn(ctx))
			.await
			.expect("test timed out");
	});
}

async fn assert_error_response(
	response: Response,
	expected_status: StatusCode,
	expected_error_code: &str,
) -> Value {
	assert_eq!(response.status(), expected_status);
	assert_eq!(
		response.headers()[reqwest::header::CONTENT_TYPE],
		"application/json"
	);
	let body = response
		.json::<Value>()
		.await
		.expect("failed to decode error response");
	assert_eq!(body["group"], "auth");
	assert_eq!(body["code"], expected_error_code);
	assert!(
		body["message"]
			.as_str()
			.is_some_and(|message| !message.is_empty())
	);
	assert!(
		body.get("metadata").is_none(),
		"authentication errors must not disclose metadata"
	);
	body
}

async fn retry_success(build: impl Fn() -> reqwest::RequestBuilder) -> Response {
	let mut last_failure = None;
	for _ in 0..100 {
		let response = build().send().await.expect("JWT issuance request failed");
		if response.status().is_success() {
			return response;
		}
		let status = response.status();
		let body = response.text().await.unwrap_or_default();
		last_failure = Some((status, body));
		tokio::time::sleep(Duration::from_millis(100)).await;
	}
	panic!("JWT issuance did not become ready: {last_failure:?}");
}
