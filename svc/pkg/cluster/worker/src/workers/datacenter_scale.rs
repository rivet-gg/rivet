use std::{cmp::Ordering, iter::{DoubleEndedIterator, Iterator}};

use chirp_worker::prelude::*;
use futures_util::{StreamExt, TryStreamExt};
use proto::backend::{self, pkg::*};

#[derive(sqlx::FromRow)]
struct ServerRow {
	server_id: Uuid,
	pool_type: i64,
	nomad_node_id: Option<String>,
	drain_ts: Option<i64>,
}

struct Server {
	server_id: Uuid,
	pool_type: backend::cluster::PoolType,
	nomad_node_id: Option<String>,
	is_draining: bool,
}

#[worker(name = "cluster-datacenter-scale")]
async fn worker(ctx: &OperationContext<cluster::msg::datacenter_scale::Message>) -> GlobalResult<()> {
	let crdb = ctx.crdb().await?;
	let datacenter_id = unwrap_ref!(ctx.datacenter_id).as_uuid();

	let ((cluster_id,), servers) = tokio::try_join!(
		sql_fetch_one!(
			[ctx, (Uuid,)]
			"
			SELECT
				cluster_id
			FROM db_cluster.datacenters
			WHERE
				datacenter_id = $1
			",
			datacenter_id,
		),
		// Get only ACTIVE servers
		sql_fetch_all!(
			[ctx, ServerRow]
			"
			SELECT
				server_id, pool_type, nomad_node_id, drain_ts
			FROM db_cluster.servers
			WHERE
				datacenter_id = $1 AND
				-- Filters out servers that are being destroyed/already destroyed
				cloud_destroy_ts IS NULL
			",
			datacenter_id,
		),
	)?;

	let servers = servers.into_iter()
	.map(|row| {
		Ok(Server {
			server_id: row.server_id,
			pool_type: unwrap!(backend::cluster::PoolType::from_i32(row.pool_type as i32)),
			nomad_node_id: row.nomad_node_id,
			is_draining: row.drain_ts.is_some(),
		})
	})
	.collect::<GlobalResult<Vec<_>>>()?;

	// TODO: Sort servers by cpu usage using cluster-datacenter-topology-get
	// servers.sort();

	// Fetch datacenter config
	let datacenter_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![datacenter_id.into()],
	})
	.await?;
	let dc = unwrap!(datacenter_res.datacenters.first());

	for pool in &dc.pools {
		scale_servers(ctx, &crdb, cluster_id, dc, &servers, pool).await?;
	}

	Ok(())
}

async fn scale_servers(
	ctx: &OperationContext<cluster::msg::datacenter_scale::Message>,
	crdb: &CrdbPool,
	cluster_id: Uuid,
	dc: &backend::cluster::Datacenter,
	servers: &[Server],
	pool: &backend::cluster::Pool,
) -> GlobalResult<()> {
	let pool_type = unwrap!(backend::cluster::PoolType::from_i32(pool.pool_type));
	let desired_count = pool.desired_count as usize;

	let servers_in_pool = servers.iter().filter(|server| server.pool_type == pool_type);
	let draining_servers = servers_in_pool
		.clone()
		.filter(|server| server.is_draining)
		.collect::<Vec<_>>();
	let active_server_count = servers_in_pool.clone().count() - draining_servers.len();

	match desired_count.cmp(&active_server_count) {
		Ordering::Less => match pool_type {
			backend::cluster::PoolType::Job => scale_down_job_servers(ctx, crdb, dc, servers_in_pool, active_server_count, pool).await?,
			backend::cluster::PoolType::Gg => scale_down_gg_servers(ctx, crdb, dc, servers_in_pool, active_server_count, pool).await?,
			backend::cluster::PoolType::Ats => scale_down_ats_servers(ctx, crdb, dc, servers_in_pool, active_server_count, pool).await?,
		},
		Ordering::Greater => {
			scale_up_servers(ctx, crdb, cluster_id, dc, draining_servers, active_server_count, pool).await?;
		}
		Ordering::Equal => {}
	}

	Ok(())
}

async fn scale_down_job_servers<'a, I: Iterator<Item = &'a Server> + Clone>(
	ctx: &OperationContext<cluster::msg::datacenter_scale::Message>,
	crdb: &CrdbPool,
	dc: &backend::cluster::Datacenter,
	servers: I,
	active_server_count: usize,
	pool: &backend::cluster::Pool,
) -> GlobalResult<()> {
	let datacenter_id = unwrap_ref!(dc.datacenter_id).as_uuid();
	let desired_count = pool.desired_count as usize;

	tracing::info!(
		?datacenter_id,
		active=%active_server_count,
		desired=%desired_count,
		"scaling down job"
	);

	let (nomad_servers, no_nomad_servers) = servers
		.clone()
		.partition::<Vec<_>, _>(|server| server.nomad_node_id.is_some());

	let destroy_count = (active_server_count - desired_count).min(no_nomad_servers.len());
	let drain_count = active_server_count - desired_count - destroy_count;

	// Destroy servers
	if destroy_count != 0 {
		tracing::info!(count=%destroy_count, "destroying servers");

		// Because servers are ordered by cpu usage, this will destroy the servers with the least cpu usage
		let destroy_candidates = no_nomad_servers.iter().rev().take(destroy_count);

		// Mark servers for destruction in db
		sql_execute!(
			[ctx, &crdb]
			"
			UPDATE db_cluster.servers
			SET cloud_destroy_ts = $2
			WHERE server_id = ANY($1)
			",
			destroy_candidates.clone()
				.map(|server| server.server_id)
				.collect::<Vec<_>>(),
			util::timestamp::now(),
		)
		.await?;

		for server in destroy_candidates {
			tracing::info!(
				server_id=%server.server_id,
				nomad_node_id=?server.nomad_node_id,
				"destroying server"
			);

			msg!([ctx] cluster::msg::server_destroy(server.server_id) {
				server_id: Some(server.server_id.into()),
			})
			.await?;
		}
	}

	// Drain servers
	if drain_count != 0 {
		tracing::info!(count=%drain_count, "draining servers");

		// Because servers are ordered by cpu usage, this will drain the servers with the least cpu usage
		let drain_candidates = nomad_servers.iter().rev().take(drain_count);

		// Mark servers as draining in db
		sql_execute!(
			[ctx, &crdb]
			"
			UPDATE db_cluster.servers
			SET drain_ts = $2
			WHERE server_id = ANY($1)
			",
			drain_candidates.clone()
				.map(|server| server.server_id)
				.collect::<Vec<_>>(),
			util::timestamp::now(),
		)
		.await?;

		for server in drain_candidates {
			tracing::info!(
				server_id=%server.server_id,
				nomad_node_id=?server.nomad_node_id,
				"draining server"
			);

			msg!([ctx] cluster::msg::server_drain(server.server_id) {
				server_id: Some(server.server_id.into()),
			})
			.await?;
		}
	}

	Ok(())
}

async fn scale_down_gg_servers<'a, I: Iterator<Item = &'a Server> + Clone>(
	ctx: &OperationContext<cluster::msg::datacenter_scale::Message>,
	crdb: &CrdbPool,
	dc: &backend::cluster::Datacenter,
	servers_in_dc: I,
	active_server_count: usize,
	pool: &backend::cluster::Pool,
) -> GlobalResult<()> {
	let datacenter_id = unwrap_ref!(dc.datacenter_id).as_uuid();
	let desired_count = pool.desired_count as usize;
	let (gg_servers, job_servers) = servers_in_dc
		.filter(|server| !matches!(server.pool_type, backend::cluster::PoolType::Ats))
		.partition::<Vec<_>, _>(|server| matches!(server.pool_type, backend::cluster::PoolType::Gg));

	tracing::info!(
		?datacenter_id,
		active=%active_server_count,
		desired=%desired_count,
		"scaling down gg"
	);

	// If there are active job servers, leave one gg server open (it will get destroyed in
	// `cluster-server-drain-complete` once the last job server is finished draining)
	let destroy_count = if job_servers.is_empty() {
		active_server_count - desired_count
	} else {
		(active_server_count - desired_count).min(active_server_count - 1)
	};

	// Destroy servers
	if destroy_count != 0 {
		tracing::info!(count=%destroy_count, "destroying servers");

		// Because servers are ordered by cpu usage, this will destroy the servers with the least cpu usage
		let destroy_candidates = gg_servers.iter().rev().take(destroy_count);

		// Mark servers for destruction in db
		sql_execute!(
			[ctx, &crdb]
			"
			UPDATE db_cluster.servers
			SET cloud_destroy_ts = $2
			WHERE server_id = ANY($1)
			",
			destroy_candidates.clone()
				.map(|server| server.server_id)
				.collect::<Vec<_>>(),
			util::timestamp::now(),
		)
		.await?;

		for server in destroy_candidates {
			tracing::info!(
				server_id=%server.server_id,
				nomad_node_id=?server.nomad_node_id,
				"destroying server"
			);

			msg!([ctx] cluster::msg::server_destroy(server.server_id) {
				server_id: Some(server.server_id.into()),
			})
			.await?;
		}
	} else if !job_servers.is_empty() {
		tracing::info!("job servers not drained yet, leaving one gg server");
	}

	Ok(())
}

async fn scale_down_ats_servers<'a, I: Iterator<Item = &'a Server> + DoubleEndedIterator + Clone>(
	ctx: &OperationContext<cluster::msg::datacenter_scale::Message>,
	crdb: &CrdbPool,
	dc: &backend::cluster::Datacenter,
	servers: I,
	active_server_count: usize,
	pool: &backend::cluster::Pool,
) -> GlobalResult<()> {
	let datacenter_id = unwrap_ref!(dc.datacenter_id).as_uuid();
	let desired_count = pool.desired_count as usize;

	tracing::info!(
		?datacenter_id,
		active=%active_server_count,
		desired=%desired_count,
		"scaling down ats"
	);

	let destroy_count = active_server_count - desired_count;

	// Destroy servers
	if destroy_count != 0 {
		tracing::info!(count=%destroy_count, "destroying servers");

		// Because servers are ordered by cpu usage, this will destroy the servers with the least cpu usage
		let destroy_candidates = servers.rev().take(destroy_count);

		// Mark servers for destruction in db
		sql_execute!(
			[ctx, &crdb]
			"
			UPDATE db_cluster.servers
			SET cloud_destroy_ts = $2
			WHERE server_id = ANY($1)
			",
			destroy_candidates.clone()
				.map(|server| server.server_id)
				.collect::<Vec<_>>(),
			util::timestamp::now(),
		)
		.await?;

		for server in destroy_candidates {
			tracing::info!(
				server_id=%server.server_id,
				nomad_node_id=?server.nomad_node_id,
				"destroying server"
			);

			msg!([ctx] cluster::msg::server_destroy(server.server_id) {
				server_id: Some(server.server_id.into()),
			})
			.await?;
		}
	}

	Ok(())
}

async fn scale_up_servers(
	ctx: &OperationContext<cluster::msg::datacenter_scale::Message>,
	crdb: &CrdbPool,
	cluster_id: Uuid,
	dc: &backend::cluster::Datacenter,
	draining_servers: Vec<&Server>,
	active_server_count: usize,
	pool: &backend::cluster::Pool,
) -> GlobalResult<()> {
	let datacenter_id = unwrap_ref!(dc.datacenter_id).as_uuid();
	let desired_count = pool.desired_count as usize;

	tracing::info!(
		?datacenter_id,
		active=%active_server_count,
		draining=%draining_servers.len(),
		desired=%desired_count,
		"scaling up"
	);

	let undrain_count = (desired_count - active_server_count).min(draining_servers.len());
	let provision_count = desired_count - active_server_count - undrain_count;

	// Undrain servers
	if undrain_count != 0 {
		tracing::info!(count=%undrain_count, "undraining servers");

		// Because servers are ordered by cpu usage, this will undrain the servers with the most cpu usage
		let undrain_candidates = draining_servers.iter().take(undrain_count);

		// Mark servers as not draining in db
		sql_execute!(
			[ctx, &crdb]
			"
			UPDATE db_cluster.servers
			SET drain_ts = NULL
			WHERE server_id = ANY($1)
			",
			undrain_candidates.clone()
				.map(|server| server.server_id)
				.collect::<Vec<_>>(),
		)
		.await?;

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
					cluster_id: Some(cluster_id.into()),
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

	Ok(())
}
