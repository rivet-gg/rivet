use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker(name = "cluster-server-destroy")]
async fn worker(ctx: &OperationContext<cluster::msg::server_destroy::Message>) -> GlobalResult<()> {
	todo!();

	Ok(())
}
