use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker(name = "linode-prebake-install-complete")]
async fn worker(ctx: &OperationContext<linode::msg::prebake_install_complete::Message>) -> GlobalResult<()> {
	todo!();

	Ok(())
}
