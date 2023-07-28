use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker_test]
async fn basic(ctx: TestCtx) {
	op!([ctx] db_query_run {

	})
	.await
	.unwrap();
}
