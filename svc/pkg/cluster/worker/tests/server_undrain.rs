use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker_test]
async fn server_undrain(ctx: TestCtx) {
	if !util::feature::server_provision() {
		return;
	}

	// msg!([ctx] cluster::msg::server_undrain() {

	// })
	// .await
	// .unwrap();

	todo!();
}
