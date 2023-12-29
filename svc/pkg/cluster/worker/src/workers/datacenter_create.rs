use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker(name = "cluster-datacenter-create")]
async fn worker(
	ctx: &OperationContext<cluster::msg::datacenter_create::Message>,
) -> GlobalResult<()> {
	let config = unwrap_ref!(ctx.config);
	let cluster_id = unwrap_ref!(config.cluster_id).as_uuid();
	let datacenter_id = unwrap_ref!(config.datacenter_id).as_uuid();

	let mut config_buf = Vec::with_capacity(config.encoded_len());
	config.encode(&mut config_buf)?;

	sql_execute!(
		[ctx]
		"
		INSERT INTO db_cluster.datacenters (
			datacenter_id,
			cluster_id,
			config,
			name_id
		)
		VALUES ($1, $2, $3, $4)
		",
		datacenter_id,
		cluster_id,
		config_buf,
		// Datacenters have a unique constraint on name ids
		&config.name_id,
	)
	.await?;

	msg!([ctx] cluster::msg::datacenter_scale(datacenter_id) {
		datacenter_id: config.datacenter_id,
	})
	.await?;

	Ok(())
}
