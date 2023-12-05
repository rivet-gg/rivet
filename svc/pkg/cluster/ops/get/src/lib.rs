use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

#[operation(name = "cluster-get")]
pub async fn handle(
	ctx: OperationContext<cluster::get::Request>,
) -> GlobalResult<cluster::get::Response> {
	let cluster_ids = ctx
		.cluster_ids
		.iter()
		.map(common::Uuid::as_uuid)
		.collect::<Vec<_>>();

	let configs = sql_fetch_all!(
		[ctx, (Vec<u8>,)]
		"
		SELECT
			config
		FROM db_cluster.clusters
		WHERE cluster_id = ANY($1)
		",
		cluster_ids
	)
	.await?;

	Ok(cluster::get::Response {
		clusters: configs
			.into_iter()
			.map(|(config_bytes,)| {
				backend::cluster::Cluster::decode(config_bytes.as_slice()).map_err(Into::into)
			})
			.collect::<GlobalResult<Vec<_>>>()?,
	})
}
