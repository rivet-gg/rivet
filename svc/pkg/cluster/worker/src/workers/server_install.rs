use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker(name = "cluster-server-install")]
async fn worker(ctx: &OperationContext<cluster::msg::server_install::Message>) -> GlobalResult<()> {
	todo!();

	Ok(())
}
