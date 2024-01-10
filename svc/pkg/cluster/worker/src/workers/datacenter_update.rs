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
	let datacenter_config = unwrap!(
		datacenter_res.datacenters.first(),
		"datacenter does not exist"
	);

	// Update config
	let mut new_config = datacenter_config.clone();

	for pool in &ctx.pools {
		// Check if pool config already exists
		if let Some(current_pool) = new_config
			.pools
			.iter_mut()
			.find(|p| p.pool_type == pool.pool_type)
		{
			// Update pool config
			current_pool.hardware = pool.hardware.clone();
			current_pool.desired_count = pool.desired_count;
		} else {
			new_config.pools.push(pool.clone());
		}
	}

	if let Some(drain_timeout) = ctx.drain_timeout {
		new_config.drain_timeout = drain_timeout;
	}

	// Encode config
	let mut config_buf = Vec::with_capacity(new_config.encoded_len());
	new_config.encode(&mut config_buf)?;

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
