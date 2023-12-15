use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker_test]
async fn empty(ctx: TestCtx) {
	op!([ctx] cluster_datacenter_topology_get {
		datacenter_ids: vec![],
	})
	.await
	.unwrap();
}
