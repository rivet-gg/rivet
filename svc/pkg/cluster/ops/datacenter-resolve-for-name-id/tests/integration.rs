use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker_test]
async fn empty(ctx: TestCtx) {
	todo!();

	// op!([ctx] cluster_datacenter_resolve_for_name_id {
	// 	cluster_ids: ,
	// 	name_ids: vec![],
	// })
	// .await
	// .unwrap();
}
