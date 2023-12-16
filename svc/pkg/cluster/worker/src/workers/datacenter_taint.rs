use std::time::Duration;

use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[worker(name = "cluster-datacenter-taint", timeout = 200)]
async fn worker(ctx: &OperationContext<cluster::msg::datacenter_taint::Message>) -> GlobalResult<()> {
	let crdb = ctx.crdb().await?;
	let datacenter_id = unwrap_ref!(ctx.datacenter_id).as_uuid();

	// Taint server records
	let server_ids = sql_fetch_all!(
		[ctx, (Uuid,), &crdb]
		"
		UPDATE db_cluster.servers
		SET taint_ts = $2
		WHERE datacenter_id = $1
		RETURNING server_id
		",
		&datacenter_id,
		util::timestamp::now(),
	)
	.await?.into_iter().map(|(server_id,)| server_id).collect::<Vec<_>>();

	// Trigger rescale
	msg!([ctx] cluster::msg::datacenter_scale(datacenter_id) {
		datacenter_id: Some(datacenter_id.into()),
	})
	.await?;

	// Poll until a job node is fully created
	loop {
		tokio::time::sleep(Duration::from_secs(10)).await;
		
		let (job_node_exists,) = sql_fetch_one!(
			[ctx, (bool,), &crdb]
			"
			SELECT EXISTS (
				SELECT 1
				FROM db_cluster.servers
				WHERE
					datacenter_id = $1 AND
					pool_type = $2 AND
					nomad_node_id IS NOT NULL AND
					taint_ts IS NULL
			)
			",
			&datacenter_id,
			backend::cluster::PoolType::Job as i32 as i64,
		)
		.await?;

		if job_node_exists {
			break;
		}
	}

	// Mark servers for destruction in db
	sql_execute!(
		[ctx, &crdb]
		"
		UPDATE db_cluster.servers
		SET cloud_destroy_ts = $2
		WHERE server_id = ANY($1)
		",
		&server_ids,
		util::timestamp::now(),
	)
	.await?;

	// Destroy all tainted servers
	for server_id in server_ids {
		msg!([ctx] cluster::msg::server_destroy(server_id) {
			server_id: Some(server_id.into()),
		})
		.await?;
	}

	msg!([ctx] cluster::msg::datacenter_taint_complete(datacenter_id) {
		datacenter_id: Some(datacenter_id.into()),
	})
	.await?;

	Ok(())
}
