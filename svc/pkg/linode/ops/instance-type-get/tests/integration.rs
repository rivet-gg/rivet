use chirp_worker::prelude::*;

#[worker_test]
async fn basic(ctx: TestCtx) {
	let res = op!([ctx] linode_instance_type_get {
		hardware_ids: vec!["g6-nanode-1".to_string()],
	})
	.await
	.unwrap();

	tracing::info!(?res);
}
