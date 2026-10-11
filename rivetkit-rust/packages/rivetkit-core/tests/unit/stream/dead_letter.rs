use super::*;
use axum::{
	Json, Router,
	extract::State,
	http::{HeaderMap, StatusCode},
	routing::put,
};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn pending_handoff_survives_failure_and_uncertain_publication_without_skipping_input() {
	#[derive(Default)]
	struct Store {
		received: Vec<Value>,
		calls: usize,
		producer: Option<String>,
	}
	let state = Arc::new(Mutex::new(Store::default()));
	async fn append(
		State(state): State<Arc<Mutex<Store>>>,
		headers: HeaderMap,
		Json(body): Json<Value>,
	) -> (StatusCode, HeaderMap) {
		assert_eq!(headers["authorization"], "Bearer process-secret");
		assert_eq!(headers["producer-epoch"], "0");
		assert_eq!(headers["producer-seq"], "0");
		let producer = headers["producer-id"].to_str().unwrap();
		let mut state = state.lock().unwrap();
		state.calls += 1;
		if let Some(previous) = &state.producer {
			assert_eq!(previous, producer);
		} else {
			state.producer = Some(producer.into());
			state.received.push(body);
		}
		let mut headers = HeaderMap::new();
		headers.insert("stream-next-offset", "001".parse().unwrap());
		(
			if state.calls == 1 {
				StatusCode::SERVICE_UNAVAILABLE
			} else {
				StatusCode::OK
			},
			headers,
		)
	}
	let app = Router::new()
		.route(
			"/streams/dlq",
			put(|| async { StatusCode::OK }).post(append),
		)
		.with_state(state.clone());
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let root = url::Url::parse(&format!(
		"http://{}/streams",
		listener.local_addr().unwrap()
	))
	.unwrap();
	let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
	let connection = Connection {
		primary: true,
		dead_letters: Default::default(),
		actor_key: Default::default(),
		subscriptions: Default::default(),
		client: Client::new(),
		root,
		token: Some("process-secret".into()),
		path: "actors/a/id".into(),
		subscription: "actor-id".into(),
	};
	let ctx = ActorContext::new("id", "a", vec![], "local");
	crate::actor::internal_storage::schema::ensure_internal_schema(ctx.sql())
		.await
		.unwrap();
	let mut checkpoint = Checkpoint::decode(r#"{"offset":"000","completed":1}"#).unwrap();
	for _ in 0..6 {
		checkpoint.failed(&anyhow::anyhow!("poison"), 100);
	}
	ctx.prepare_dead_letter(
		&mut checkpoint,
		"checkpoint",
		"primary",
		"actors/a/id",
		"dlq".into(),
		&json!({"id":8}),
	)
	.await
	.unwrap();
	assert!(
		!ctx.deliver_dead_letter(&connection, &mut checkpoint, "checkpoint")
			.await
			.unwrap()
	);
	let raw = internal_storage::load_meta_text(ctx.sql(), "checkpoint")
		.await
		.unwrap()
		.unwrap();
	let mut restored = Checkpoint::decode(&raw).unwrap();
	assert!(
		restored
			.dead_letter
			.as_ref()
			.unwrap()
			.delivery_error
			.is_some()
	);
	assert_eq!(restored.completed, 1);
	assert_eq!(restored.offset, "000");
	assert_eq!(restored.failure.as_ref().unwrap().attempts, 6);
	assert!(
		ctx.deliver_dead_letter(&connection, &mut restored, "checkpoint")
			.await
			.unwrap()
	);
	// Simulate a crash after acceptance but before checkpoint advancement.
	let mut restored = Checkpoint::decode(&raw).unwrap();
	assert!(
		ctx.deliver_dead_letter(&connection, &mut restored, "checkpoint")
			.await
			.unwrap()
	);
	assert_eq!(restored.completed, 1);
	let stored = state.lock().unwrap();
	assert_eq!(stored.calls, 3);
	assert_eq!(stored.received.len(), 1);
	assert_eq!(stored.received[0][0]["body"], json!({"id":8}));
	server.abort();
}
