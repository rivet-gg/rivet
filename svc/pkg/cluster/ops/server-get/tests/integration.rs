use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker_test]
async fn empty(ctx: TestCtx) {
	op!([ctx] cluster_server_get {
		server_ids: vec![todo!()],
	})
	.await
	.unwrap();
}
