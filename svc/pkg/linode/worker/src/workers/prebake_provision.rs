use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker(name = "linode-prebake-provision")]
async fn worker(ctx: &OperationContext<linode::msg::prebake_provision::Message>) -> GlobalResult<()> {
	todo!();

	Ok(())
}
