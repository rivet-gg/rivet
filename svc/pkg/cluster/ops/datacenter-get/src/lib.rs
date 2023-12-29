use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

#[operation(name = "cluster-datacenter-get")]
pub async fn handle(
	ctx: OperationContext<cluster::datacenter_get::Request>,
) -> GlobalResult<cluster::datacenter_get::Response> {
	let datacenter_ids = ctx
		.datacenter_ids
		.iter()
		.map(common::Uuid::as_uuid)
		.collect::<Vec<_>>();

	let configs = sql_fetch_all!(
		[ctx, (Vec<u8>,)]
		"
		SELECT
			config
		FROM db_cluster.datacenters
		WHERE datacenter_id = ANY($1)
		",
		datacenter_ids
	)
	.await?;

	Ok(cluster::datacenter_get::Response {
		datacenters: configs
			.into_iter()
			.map(|(config_bytes,)| {
				backend::cluster::Datacenter::decode(config_bytes.as_slice()).map_err(Into::into)
			})
			.collect::<GlobalResult<Vec<_>>>()?,
	})
}
