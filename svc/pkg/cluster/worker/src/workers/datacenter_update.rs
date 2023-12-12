use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker(name = "cluster-datacenter-update")]
async fn worker(ctx: &OperationContext<cluster::msg::datacenter_update::Message>) -> GlobalResult<()> {
	let config = unwrap_ref!(ctx.config);
	let cluster_id = unwrap_ref!(config.cluster_id).as_uuid();
	let datacenter_id = unwrap_ref!(config.datacenter_id).as_uuid();

	let mut config_buf = Vec::with_capacity(config.encoded_len());
	config.encode(&mut config_buf)?;
	
	sql_execute!(
		[ctx]
		"
		UPDATE db_cluster.datacenters
		SET config = $2
		WHERE datacenter_id = $1
		",
		datacenter_id,
		config_buf,
	)
	.await?;

	msg!([ctx] cluster::msg::update(cluster_id) {
		cluster_id: config.cluster_id,
	}).await?;

	Ok(())
}
