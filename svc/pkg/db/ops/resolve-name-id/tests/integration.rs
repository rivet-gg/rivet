use chirp_worker::prelude::*;

#[worker_test]
async fn basic(ctx: TestCtx) {
	op!([ctx] db_resolve_name_id {
		name_ids: Vec::new()
	})
	.await
	.unwrap();
}
