use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker(name = "cluster-server-drain")]
async fn worker(ctx: &OperationContext<cluster::msg::server_drain::Message>) -> GlobalResult<()> {
	todo!();

	Ok(())
}
