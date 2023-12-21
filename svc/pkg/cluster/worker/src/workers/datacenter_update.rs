use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker(name = "cluster-datacenter-update")]
async fn worker(
	ctx: &OperationContext<cluster::msg::datacenter_update::Message>,
) -> GlobalResult<()> {
	let datacenter_id = unwrap_ref!(ctx.datacenter_id).as_uuid();

	let datacenter_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![datacenter_id.into()],
	})
	.await?;
	let datacenter = unwrap!(
		datacenter_res.datacenters.first(),
		"datacenter does not exist"
	);

	// Update config
	let mut config = datacenter.clone();
	config.pools = ctx.pools.clone();
	if let Some(drain_timeout) = ctx.drain_timeout {
		config.drain_timeout = drain_timeout;
	}

	// Encode config
	let mut config_buf = Vec::with_capacity(config.encoded_len());
	config.encode(&mut config_buf)?;

	// Write config
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

	msg!([ctx] cluster::msg::datacenter_scale(datacenter_id) {
		datacenter_id: ctx.datacenter_id,
	})
	.await?;

	Ok(())
}
