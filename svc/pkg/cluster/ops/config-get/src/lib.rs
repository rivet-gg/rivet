use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

#[operation(name = "cluster-config-get")]
pub async fn handle(
	ctx: OperationContext<cluster::config_get::Request>,
) -> GlobalResult<cluster::config_get::Response> {
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
		FROM db_cluster.cluster_config
		WHERE cluster_id = ANY($1)
		",
		cluster_ids
	)
	.await?;

	Ok(cluster::config_get::Response {
		configs: configs
			.into_iter()
			.map(|(config_bytes,)| {
				backend::cluster::Cluster::decode(config_bytes.as_slice()).map_err(Into::into)
			})
			.collect::<GlobalResult<Vec<_>>>()?,
	})
}
