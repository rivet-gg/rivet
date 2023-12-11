use std::{cmp::Ordering, iter::Iterator};

use chirp_worker::prelude::*;
use futures_util::{StreamExt, TryStreamExt};
use proto::backend::{self, pkg::*};

#[derive(sqlx::FromRow)]
struct ServerRow {
	server_id: Uuid,
	datacenter_id: Uuid,
	pool_type: i64,
	nomad_node_id: Option<String>,
	drain_ts: Option<i64>,
}

struct Server {
	server_id: Uuid,
	datacenter_id: Uuid,
	pool_type: backend::cluster::PoolType,
	nomad_node_id: Option<String>,
	is_draining: bool,
}

#[worker(name = "cluster-scale")]
async fn worker(ctx: &OperationContext<cluster::msg::update::Message>) -> GlobalResult<()> {
	let crdb = ctx.crdb().await?;
	let cluster = unwrap_ref!(ctx.cluster);
	let cluster_id = unwrap!(cluster.cluster_id).as_uuid();

	// Get ACTIVE servers
	let servers = sql_fetch_all!(
		[ctx, ServerRow]
		"
		SELECT
			server_id, datacenter_id, pool_type, nomad_node_id, drain_ts
		FROM db_cluster.servers
		WHERE
			cluster_id = $1 AND
			-- Filters out servers that are being destroyed/already destroyed
			cloud_destroy_ts IS NULL
		",
		cluster_id
	)
	.await?
	.into_iter()
	.map(|row| {
		Ok(Server {
			server_id: row.server_id,
			datacenter_id: row.datacenter_id,
			pool_type: unwrap!(backend::cluster::PoolType::from_i32(row.pool_type as i32)),
			nomad_node_id: row.nomad_node_id,
			is_draining: row.drain_ts.is_some(),
		})
	})
	.collect::<GlobalResult<Vec<_>>>()?;

	// Fetch datacenter config
	let cluster_datacenter_list_res = op!([ctx] cluster_datacenter_list {
		cluster_ids: vec![cluster_id.into()],
	}).await?;
	let datacenter_ids = &unwrap!(cluster_datacenter_list_res.clusters.first()).datacenter_ids;

	let datacenters_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: datacenter_ids.clone(),
	}).await?;

	// Scale all datacenters
	for dc in &datacenters_res.datacenters {
		let datacenter_id = unwrap!(dc.datacenter_id).as_uuid();
		let servers_in_dc = servers
			.iter()
			.filter(|server| server.datacenter_id == datacenter_id);

		for pool in &dc.pools {
			scale_servers(
				ctx,
				&crdb,
				cluster,
				dc,
				servers_in_dc.clone(),
				pool,
			)
			.await?;
		}
	}

	Ok(())
}

async fn scale_servers<'a, I: Iterator<Item = &'a Server> + Clone>(
	ctx: &OperationContext<cluster::msg::update::Message>,
	crdb: &CrdbPool,
	cluster: &backend::cluster::Cluster,
	dc: &backend::cluster::Datacenter,
	servers_in_dc: I,
	pool: &backend::cluster::Pool,
) -> GlobalResult<()> {
	let cluster_id = unwrap!(cluster.cluster_id).as_uuid();
	let datacenter_id = unwrap!(dc.datacenter_id).as_uuid();
	let pool_type = unwrap!(backend::cluster::PoolType::from_i32(pool.pool_type));
	let desired_count = pool.desired_count as usize;

	let job_servers = servers_in_dc
		.filter(|server| server.pool_type == pool_type);
	let draining_servers = job_servers.clone().filter(|server| server.is_draining).collect::<Vec<_>>();
	let active_server_count = job_servers.count() - draining_servers.len();

	match desired_count.cmp(&active_server_count) {
		Ordering::Greater => {
			tracing::info!(
				?datacenter_id,
				active=%active_server_count,
				draining=%draining_servers.len(),
				desired=%desired_count,
				"scaling down"
			);

			match pool_type {
				backend::cluster::PoolType::Job => todo!(),
				backend::cluster::PoolType::Gg => todo!(),
				backend::cluster::PoolType::Ats => todo!(),
			}
		}
		Ordering::Less => {
			tracing::info!(
				?datacenter_id,
				active=%active_server_count,
				draining=%draining_servers.len(),
				desired=%desired_count,
				"scaling up"
			);
	
			let undrain_count = (desired_count - active_server_count).min(draining_servers.len());
			let provision_count = desired_count - undrain_count - active_server_count;

			// Undrain servers
			if undrain_count != 0 {
				// Mark servers as not draining in db
				sql_execute!(
					[ctx]
					"
					UPDATE db_cluster.servers
					SET drain_ts = NULL
					WHERE server_id = ANY($1)
					",
					draining_servers
						.iter()
						.map(|server| server.server_id)
						.collect::<Vec<_>>(),
				)
				.await?;

				// TODO: Sort by cpu usage (using cluster-topology-get), undrain servers with most cpu usage
				let undrain_candidates = draining_servers.iter().take(undrain_count);

				for draining_server in undrain_candidates {
					tracing::info!(
						server_id=%draining_server.server_id,
						nomad_node_id=?draining_server.nomad_node_id,
						"undraining server"
					);

					msg!([ctx] cluster::msg::server_undrain(draining_server.server_id) {
						server_id: Some(draining_server.server_id.into()),
					})
					.await?;
				}
			}

			// Create new servers
			if provision_count != 0 {
				tracing::info!(count=%provision_count, "provisioning servers");

				futures_util::stream::iter(0..provision_count)
					.map(|_| async {
						let server_id = Uuid::new_v4();
		
						// Write new server to db
						sql_execute!(
							[ctx, &crdb]
							"
							INSERT INTO db_cluster.servers (
								server_id,
								datacenter_id,
								cluster_id,
								pool_type,
								create_ts
							)
							VALUES ($1, $2, $3, $4, $5)
							",
							server_id,
							datacenter_id,
							cluster_id,
							pool.pool_type as i64,
							util::timestamp::now(),
						)
						.await?;
		
						msg!([ctx] cluster::msg::server_provision(server_id) {
							cluster_id: cluster.cluster_id,
							datacenter_id: dc.datacenter_id,
							server_id: Some(server_id.into()),
							pool_type: pool.pool_type,
							provider: dc.provider,
						})
						.await?;
		
						GlobalResult::Ok(())
					})
					.buffer_unordered(16)
					.try_collect::<Vec<_>>()
					.await?;
			}
		}
		Ordering::Equal => {}
	}

	Ok(())
}
