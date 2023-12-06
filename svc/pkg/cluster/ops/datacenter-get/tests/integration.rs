use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker_test]
async fn empty(ctx: TestCtx) {
	op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![todo!()],
	})
	.await
	.unwrap();
}
