use super::*;
use axum::{Json, Router, routing::get};
use ed25519_dalek::{Signer, SigningKey};

#[tokio::test]
async fn stream_wake_requires_trusted_signature_freshness_and_destination_binding() {
	let signer = SigningKey::from_bytes(&[23; 32]);
	let public = URL_SAFE_NO_PAD.encode(signer.verifying_key().to_bytes());
	let jwks = json!({"keys":[{"kid":"test-key","kty":"OKP","crv":"Ed25519","x":public}]});
	let app = Router::new().route(
		"/streams/__ds/jwks.json",
		get(move || async move { Json(jwks) }),
	);
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let root = format!("http://{}/streams", listener.local_addr().unwrap());
	let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
	let connection = Connection {
		dead_letters: Default::default(),
		primary: true,
		actor_key: vec![],
		subscriptions: BTreeMap::new(),
		client: Client::new(),
		root: url::Url::parse(&root).unwrap(),
		token: None,
		path: "actors/worker/actor-1".into(),
		subscription: "actor-actor-1".into(),
	};
	let wake = json!({"subscription_id":"actor-actor-1","wake_id":"wake-1","generation":1,
		"streams":[{"path":"actors/worker/actor-1","acked_offset":"0000","tail_offset":"0001"}],
		"callback_url":format!("{root}/__ds/subscriptions/actor-actor-1/callback"),"callback_token":"capability"});
	let now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs();
	let sign = |key: &SigningKey, time: u64, value: &Value| {
		let body = serde_json::to_vec(value).unwrap();
		let mut signed = format!("{time}.").into_bytes();
		signed.extend_from_slice(&body);
		let signature = URL_SAFE_NO_PAD.encode(key.sign(&signed).to_bytes());
		(format!("t={time},kid=test-key,ed25519={signature}"), body)
	};
	let (header, body) = sign(&signer, now, &wake);
	assert!(connection.verify(&header, &body).await.is_ok());
	// Identical wakes are valid redeliveries; the durable checkpoint handles duplicates.
	assert!(connection.verify(&header, &body).await.is_ok());
	assert!(
		connection
			.verify(&format!("{header},kid=test-key"), &body)
			.await
			.is_err()
	);
	assert!(connection.verify(&header, b"{}").await.is_err());
	for time in [now - 301, now + 301] {
		let (header, body) = sign(&signer, time, &wake);
		assert!(connection.verify(&header, &body).await.is_err());
	}
	let (header, body) = sign(&SigningKey::from_bytes(&[42; 32]), now, &wake);
	assert!(connection.verify(&header, &body).await.is_err());
	for field in [
		"subscription",
		"stream",
		"callback",
		"multiple",
		"generation",
	] {
		let mut wrong = wake.clone();
		match field {
			"subscription" => wrong["subscription_id"] = json!("actor-other"),
			"stream" => wrong["streams"][0]["path"] = json!("actors/worker/other"),
			"callback" => wrong["callback_url"] = json!("https://other.gameinc.io/callback"),
			"multiple" => {
				wrong["streams"]
					.as_array_mut()
					.unwrap()
					.push(wake["streams"][0].clone());
			}
			_ => wrong["generation"] = json!(-1),
		}
		let (header, body) = sign(&signer, now, &wrong);
		assert!(
			connection.verify(&header, &body).await.is_err(),
			"accepted wrong {field}"
		);
	}
	server.abort();
}

#[test]
fn named_wakes_bind_all_sources_to_handler_and_actor_key() {
	let connection = Connection {
		dead_letters: Default::default(),
		client: Client::new(),
		root: url::Url::parse("https://streams.gameinc.io/streams").unwrap(),
		token: None,
		primary: false,
		path: "actors/customer/actor-1".into(),
		subscription: "actor-actor-1".into(),
		actor_key: vec![crate::types::ActorKeySegment::String("acme".into())],
		subscriptions: BTreeMap::from([(
			"orders".into(),
			NamedStreamSubscription {
				stream: "/orders/:tenant/:source".into(),
				actor_key: vec![":tenant".into()],
			},
		)]),
	};
	let mut wake: Wake=serde_json::from_value(json!({"subscription_id":"actor-actor-1-orders","wake_id":"w","generation":1,"streams":[{"path":"orders/acme/a","acked_offset":"0","tail_offset":"1"},{"path":"orders/acme/b","acked_offset":"0","tail_offset":"1"}],"callback_url":"","callback_token":""})).unwrap();
	assert_eq!(connection.validate_destination(&wake).unwrap(), "orders");
	wake.streams[1].path = "orders/other/b".into();
	assert!(connection.validate_destination(&wake).is_err());
	wake.streams[1].path = "orders/acme/a".into();
	assert!(connection.validate_destination(&wake).is_err());
	wake.streams.pop();
	wake.subscription_id = "actor-actor-1-missing".into();
	assert!(connection.validate_destination(&wake).is_err());
	assert_eq!(
		connection
			.source_request(Method::GET, "orders/acme/a%2Fb")
			.unwrap()
			.build()
			.unwrap()
			.url()
			.path(),
		"/streams/orders/acme/a%2Fb"
	);
	assert!(connection.source_request(Method::GET, "../escape").is_err());
}

#[test]
fn caller_append_never_adds_the_process_credential() {
	let connection = Connection {
		dead_letters: Default::default(),
		primary: true,
		actor_key: Default::default(),
		subscriptions: Default::default(),
		client: Client::new(),
		root: url::Url::parse("https://streams.gameinc.io/streams").unwrap(),
		token: Some("process-secret".into()),
		path: "actors/worker/id".into(),
		subscription: "actor-id".into(),
	};
	let request = connection
		.source_request_with_token(Method::POST, &connection.path, Some("caller-token"))
		.unwrap()
		.build()
		.unwrap();
	let values = request
		.headers()
		.get_all("authorization")
		.iter()
		.collect::<Vec<_>>();
	assert_eq!(values.len(), 1);
	assert_eq!(values[0], "Bearer caller-token");
	let read = connection
		.source_request(Method::GET, &connection.path)
		.unwrap()
		.build()
		.unwrap();
	assert_eq!(read.headers()["authorization"], "Bearer process-secret");
}

#[test]
fn failures_preserve_budget_deadline_and_pause_across_checkpoint_reload() {
	let legacy = r#"{"offset":"0007","completed":1}"#;
	let mut checkpoint = Checkpoint::decode(legacy).unwrap();
	for attempt in 1..=6 {
		checkpoint.failed(&anyhow::anyhow!("poison record"), 10_000 + attempt as i64);
		let encoded = checkpoint.encode().unwrap();
		assert!(encoded.starts_with("bare:"));
		checkpoint = Checkpoint::decode(&encoded).unwrap();
		let failure = checkpoint.failure.as_ref().unwrap();
		assert_eq!(failure.attempts, attempt);
		assert_eq!(failure.first_failed_at, 10_001);
		assert_eq!(failure.paused, attempt == 6);
		assert_eq!(checkpoint.offset, "0007");
		assert_eq!(checkpoint.completed, 1);
		if attempt == 1 {
			assert_eq!(failure.next_attempt_at, 11_001);
		}
		if attempt == 5 {
			assert_eq!(failure.next_attempt_at, 26_005);
		}
	}
	let mut restored = Checkpoint::decode(&checkpoint.encode().unwrap()).unwrap();
	restored.failure = None; // Explicit resume retains position and resets only the budget.
	restored.failed(&anyhow::anyhow!("still failing"), 99_000);
	assert_eq!(restored.failure.unwrap().attempts, 1);
	assert!(Checkpoint::decode("bare:invalid").is_err());
}

#[test]
fn checkpoint_upgrade_and_pending_handoff_preserve_payload_and_producer_identity() {
	let bytes = StreamCheckpoint::wrap_latest(wire::Data {
		offset: "001".into(),
		completed: 2,
		failure: None,
		dead_letter: None,
	})
	.serialize_with_embedded_version(1)
	.unwrap();
	let mut checkpoint =
		Checkpoint::decode(&format!("bare:{}", URL_SAFE_NO_PAD.encode(bytes))).unwrap();
	assert!(checkpoint.dead_letter.is_none());
	checkpoint.dead_letter = Some(wire::DeadLetter {
		stream: "dead-letters/a".into(),
		producer_id: "stable".into(),
		envelope: br#"{"body":{"id":8}}"#.to_vec(),
		delivery_error: Some("DS unavailable".into()),
	});
	let restored = Checkpoint::decode(&checkpoint.encode().unwrap()).unwrap();
	assert_eq!(restored.offset, "001");
	assert_eq!(restored.completed, 2);
	let pending = restored.dead_letter.unwrap();
	assert_eq!(pending.producer_id, "stable");
	assert_eq!(pending.envelope, br#"{"body":{"id":8}}"#);
	assert_eq!(pending.delivery_error.as_deref(), Some("DS unavailable"));
	assert!(
		StreamCheckpoint::wrap_latest(wire::Data {
			offset: "001".into(),
			completed: 2,
			failure: None,
			dead_letter: Some(pending)
		})
		.serialize_with_embedded_version(1)
		.is_err()
	);
}
