//! End-to-end purpose isolation and DS-compatible signing through a follower API.
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
#[path = "common/api/mod.rs"]
mod api;
#[path = "common/ctx.rs"]
mod ctx;

async fn success(request: reqwest::RequestBuilder) -> Value {
	let response = request.send().await.unwrap();
	let status = response.status();
	let body: Value = response.json().await.unwrap();
	assert!(status.is_success(), "{status}: {body}");
	body
}

#[test]
fn follower_issues_downstream_tokens_and_signs_with_separate_namespace_keys() {
	tokio::runtime::Runtime::new().unwrap().block_on(async {
        let admin = format!("signing-test-{}", rand::random::<u64>());
        let ctx = ctx::TestCtx::new_with_opts(ctx::TestOpts::new(2).with_auth_admin_token(admin.clone()).with_timeout(180)).await.unwrap();
        tokio::time::timeout(Duration::from_secs(180), async {
            let client = Client::new();
            let base = format!("http://127.0.0.1:{}", ctx.get_dc(2).guard_port());
            let name = format!("signing-{}", rand::random::<u32>());
            let ns = api::peer::namespaces_create(ctx.leader_dc().api_peer_port(), rivet_api_peer::namespaces::CreateRequest { name: name.clone(), display_name: "Signing Test".into() }).await.unwrap().namespace.namespace_id;
            let manifest = json!({
                "rules": {
                    "signing-app": {"namespace": {"id": ns}, "resource": "application_token", "target": "any", "operations": ["create"]},
                    "signing-webhook": {"namespace": {"id": ns}, "resource": "webhook_signing", "target": "any", "operations": ["create"]}
                },
                "policies": {"signing": {"rules": ["signing-app", "signing-webhook"]}},
                "roles": {"signing": {"policies": ["signing"]}}
            });
            success(client.post(format!("{base}/acl/manifest")).bearer_auth(&admin).json(&manifest)).await;
            let acl = success(client.post(format!("{base}/acl/tokens")).bearer_auth(&admin).json(&json!({"roles": ["signing"], "duration": 120}))).await;
            let token = acl["token"].as_str().unwrap();
            let response = client.post(format!("{base}/auth/webhooks/ensure")).bearer_auth(&admin).send().await.unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN, "wildcard admin must not choose a namespace");

            let missing = client.get(format!("{base}/namespaces/{ns}/auth/webhooks/jwks.json")).send().await.unwrap();
            assert_eq!(missing.status(), StatusCode::NOT_FOUND);
            assert_eq!(missing.json::<Value>().await.unwrap()["code"], "not_found");
            let metadata = success(client.post(format!("{base}/auth/webhooks/ensure")).bearer_auth(token)).await;
            let kid = metadata["kid"].as_str().unwrap();
            assert!(kid.starts_with("ds_"));
            let jwks_url = format!("{base}/namespaces/{ns}/auth/webhooks/jwks.json");
            let response = client.get(&jwks_url).send().await.unwrap();
            assert_eq!(response.headers()["cache-control"], "no-store");
            let jwks: Value = response.json().await.unwrap();
            let key = jwks["keys"].as_array().unwrap().iter().find(|k| k["kid"] == kid).unwrap();
            assert_eq!(key.as_object().unwrap().len(), 6);
            let public: [u8;32] = URL_SAFE_NO_PAD.decode(key["x"].as_str().unwrap()).unwrap().try_into().unwrap();
            let payload = vec![0,255,b'\r',b'\n',b'{',b'}'];
            let signed = success(client.post(format!("{base}/auth/webhooks/sign")).bearer_auth(token).header("content-type", "application/octet-stream").body(payload.clone())).await;
            let mut message = format!("{}.", signed["timestamp"].as_str().unwrap()).into_bytes(); message.extend_from_slice(&payload);
            let sig = ed25519_dalek::Signature::from_slice(&URL_SAFE_NO_PAD.decode(signed["signature"].as_str().unwrap()).unwrap()).unwrap();
            ed25519_dalek::VerifyingKey::from_bytes(&public).unwrap().verify_strict(&message, &sig).unwrap();
            let issued = success(client.post(format!("{base}/auth/application-tokens")).bearer_auth(token).json(&json!({"audience":"durable-streams", "purpose":"durable-streams.callback", "duration":86400, "claims":{"durable_streams":{"root":"https://streams.gameinc.io/streams", "subscription_id":"sub-1", "wake_id":"w-1", "generation":7, "operations":["callback"]}}}))).await;
            assert!(issued["expires_ts"].as_i64().unwrap() <= acl["expires_ts"].as_i64().unwrap());
            let app = issued["token"].as_str().unwrap();
            let parts: Vec<_> = app.split('.').collect();
            let header: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[0]).unwrap()).unwrap();
            let claims: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[1]).unwrap()).unwrap();
            assert_eq!(header["typ"], "rivet-app+jwt");
            assert_ne!(header["kid"], kid);
            assert_eq!(claims["rivet_ns"], ns.to_string());
            assert!(claims.get("rivet_grants").is_none());
            for path in ["/auth/tokens/inspect", "/acl/tokens/inspect"] {
                let response = client.get(format!("{base}{path}")).bearer_auth(app).send().await.unwrap();
                assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            }
            let response = client.get(format!("{base}/gateway/no-actor/ping")).query(&[("rvt-namespace", name.as_str()),("rvt-method", "get"),("rvt-key","unused"),("rvt-token",app)]).send().await.unwrap();
            assert_eq!(response.json::<Value>().await.unwrap()["code"], "invalid_token");
            let mut ws_url = url::Url::parse(&format!("ws://127.0.0.1:{}/gateway/no-actor/socket", ctx.get_dc(2).guard_port())).unwrap();
            ws_url.query_pairs_mut().extend_pairs([("rvt-namespace",name.as_str()),("rvt-method","get"),("rvt-key","unused"),("rvt-token",app)]);
            use tokio_tungstenite::tungstenite::client::IntoClientRequest;
            let mut ws_request = ws_url.as_str().into_client_request().unwrap();
            ws_request.headers_mut().insert("Sec-WebSocket-Protocol", "rivet".parse().unwrap());
            let (mut socket, _) = tokio_tungstenite::connect_async(ws_request).await.unwrap();
            use futures_util::StreamExt;
            let rejection = tokio::time::timeout(Duration::from_secs(5), socket.next()).await.unwrap().unwrap().unwrap();
            match rejection {
                tokio_tungstenite::tungstenite::Message::Close(Some(frame)) => assert!(frame.reason.starts_with("auth.invalid_token#")),
                _ => panic!("application credential must close without an actor session"),
            }
            // Management uses independently authorized namespace scope; signing token cannot rotate.
            let management = format!("{base}/namespaces/{name}/auth/webhook-keys");
            let status = success(client.get(&management).bearer_auth(&admin)).await;
            let rotate = json!({"expected_generation":status["generation"]});
            assert_eq!(client.post(format!("{management}/rotate")).bearer_auth(token).json(&rotate).send().await.unwrap().status(), StatusCode::FORBIDDEN);
            let staged = success(client.post(format!("{management}/rotate")).bearer_auth(&admin).json(&rotate)).await;
            assert_eq!(staged["active_kid"], kid);
            assert!(staged["pending_kid"].is_string());
            let stale = client.post(format!("{management}/rotate")).bearer_auth(&admin).json(&rotate).send().await.unwrap();
            assert_eq!(stale.status(), StatusCode::CONFLICT);
            assert_eq!(stale.json::<Value>().await.unwrap()["code"], "conflict");
            let invalid = client.post(format!("{management}/revoke")).bearer_auth(&admin).json(&json!({"request_id":uuid::Uuid::new_v4().to_string(),"expected_generation":staged["generation"],"kids":["ds_unknown"]})).send().await.unwrap();
            assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
            assert_eq!(invalid.json::<Value>().await.unwrap()["code"], "invalid_input");
            let revoke = json!({"request_id":uuid::Uuid::new_v4().to_string(), "expected_generation":staged["generation"], "kids":[kid]});
            let revoked = success(client.post(format!("{management}/revoke")).bearer_auth(&admin).json(&revoke)).await;
            assert_ne!(revoked["active_kid"], kid);
            assert_eq!(success(client.post(format!("{management}/revoke")).bearer_auth(&admin).json(&revoke)).await, revoked);
            let after = success(client.get(&jwks_url)).await;
            assert!(!after["keys"].as_array().unwrap().iter().any(|key| key["kid"] == kid));
            let streams: Vec<_> = (0..4096).map(|i| json!({"stream":format!("{i:04}/{}", "%C3%A9".repeat(1020)),"offset":"18446744073709551615","generation":18446744073709551615u64})).collect();
            let snapshot = serde_json::to_vec(&json!({"streams":streams,"capability":"x".repeat(6144)})).unwrap();
            assert!(snapshot.len() < 32*1024*1024);
            for (label, payload, samples) in [("1k",vec![b'x';1024],15), ("1m",vec![b'x';1024*1024],15), ("ds4096",snapshot,3)] {
                let mut timings = Vec::new();
                for _ in 0..samples {
                    let start = Instant::now();
                    success(client.post(format!("{base}/auth/webhooks/sign")).bearer_auth(token).header("content-type", "application/octet-stream").body(payload.clone())).await;
                    timings.push(start.elapsed().as_secs_f64());
                }
                timings.sort_by(f64::total_cmp);
                println!("SIGNING_BENCH label={label} bytes={} samples={samples} p50={} p95={} operations_per_second={}", payload.len(), timings[samples/2], timings[(samples*95/100).min(samples-1)], samples as f64 / timings.iter().sum::<f64>());
            }
            let mut timings = Vec::new();
            for _ in 0..30 {
                let start = Instant::now();
                success(client.post(format!("{base}/auth/application-tokens")).bearer_auth(token).json(&json!({"audience":"durable-streams", "purpose":"durable-streams.callback", "claims":{"app_data":{"generation":1}}}))).await;
                timings.push(start.elapsed().as_secs_f64());
            }
            timings.sort_by(f64::total_cmp);
            println!("SIGNING_BENCH label=jwt samples=30 p50={} p95={} operations_per_second={}", timings[15],timings[28],30.0/timings.iter().sum::<f64>());
            // Both public datacenters spend the same durable namespace operation budget.
            let leader_base = format!("http://127.0.0.1:{}",ctx.leader_dc().guard_port());
            // 35 attempts were spent above. Leave exactly one admission, then race two DCs.
            for i in 0..24 {
                let endpoint = if i % 2 == 0 { &base } else { &leader_base };
                success(client.post(format!("{endpoint}/auth/webhooks/ensure")).bearer_auth(token)).await;
            }
            let (left,right) = tokio::join!(
                client.post(format!("{base}/auth/webhooks/ensure")).bearer_auth(token).send(),
                client.post(format!("{leader_base}/auth/webhooks/ensure")).bearer_auth(token).send()
            );
            let mut statuses = [left.unwrap().status().as_u16(),right.unwrap().status().as_u16()];
            statuses.sort(); assert_eq!(statuses, [200,429], "only one concurrent contender may spend the last admission");
            for endpoint in [&base,&leader_base] {
                let response = client.post(format!("{endpoint}/auth/webhooks/ensure")).bearer_auth(token).send().await.unwrap();
                assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
                assert_eq!(response.headers()["retry-after"], "60");
            }
        }).await.unwrap();
    });
}
