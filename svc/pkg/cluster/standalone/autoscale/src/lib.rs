use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
use std::collections::HashMap;

lazy_static::lazy_static! {
	static ref SERVER_PROVISION_MARGIN: u64 = util::env::var("SERVER_PROVISION_MARGIN").unwrap()
		.parse()
		.unwrap();
}

#[tracing::instrument(skip_all)]
pub async fn run_from_env(pools: rivet_pools::Pools) -> GlobalResult<()> {
	let client = chirp_client::SharedClient::from_env(pools.clone())?.wrap_new("cluster-autoscale");
	let cache = rivet_cache::CacheInner::from_env(pools.clone())?;
	let ctx = OperationContext::new(
		"cluster-autoscale".into(),
		std::time::Duration::from_secs(60),
		rivet_connection::Connection::new(client, pools, cache),
		Uuid::new_v4(),
		Uuid::new_v4(),
		util::timestamp::now(),
		util::timestamp::now(),
		(),
		Vec::new(),
	);

	// Fetch all datacenters and their servers
	let (datacenter_rows, job_server_rows) = tokio::try_join!(
		sql_fetch_all!(
			[ctx, (Uuid,)]
			"
			SELECT datacenter_id
			FROM db_cluster.datacenters
			",
		),
		sql_fetch_all!(
			[ctx, (Uuid, Option<i64>)]
			"
			SELECT datacenter_id, memory
			FROM db_cluster.servers
			WHERE
				pool_type = $1 AND
				cloud_destroy_ts IS NULL AND
				taint_ts IS NULL
			",
			backend::cluster::PoolType::Job as i32 as i64
		)
	)?;

	let datacenter_ids = datacenter_rows
		.into_iter()
		.map(|(datacenter_id,)| datacenter_id.into())
		.collect::<Vec<_>>();

	let (datacenters_res, topologies_res) = tokio::try_join!(
		op!([ctx] cluster_datacenter_get {
			datacenter_ids: datacenter_ids.clone(),
		}),
		op!([ctx] cluster_datacenter_topology_get {
			datacenter_ids: datacenter_ids,
		}),
	)?;

	// Get all hardware types
	let hardware = datacenters_res
		.datacenters
		.iter()
		.map(|dc| {
			let datacenter_id = unwrap_ref!(dc.datacenter_id).as_uuid();
			let job_pool = unwrap!(
				dc.pools
					.iter()
					.find(|pool| pool.pool_type == backend::cluster::PoolType::Job as i32),
				"no job pool"
			);
			let hardware = unwrap!(job_pool.hardware.first(), "no hardware")
				.provider_hardware
				.clone();

			Ok((datacenter_id, hardware))
		})
		.collect::<GlobalResult<Vec<_>>>()?;

	// Fetch hardware info
	let instance_types_res = op!([ctx] linode_instance_type_get {
		// TODO: Filter duplicates
		hardware_ids: hardware
			.iter()
			.map(|(_, hardware)| hardware.clone())
			.collect::<Vec<_>>(),
	})
	.await?;

	// We assume a server has this default memory amount (memory of the first hardware in the list) before
	// it is provisioned
	let default_memory = hardware
		.into_iter()
		.map(|(datacenter_id, hardware)| {
			let instance_type = unwrap!(instance_types_res
				.instance_types
				.iter()
				.find(|hw| hw.hardware_id == hardware));

			Ok((datacenter_id, instance_type.memory))
		})
		.collect::<GlobalResult<HashMap<_, _>>>()?;

	for datacenter in &datacenters_res.datacenters {
		let datacenter_id = unwrap_ref!(datacenter.datacenter_id).as_uuid();
		let topology = unwrap!(topologies_res
			.datacenters
			.iter()
			.find(|topo| topo.datacenter_id == datacenter.datacenter_id));
		let job_pool = unwrap!(
			datacenter
				.pools
				.iter()
				.find(|pool| pool.pool_type == backend::cluster::PoolType::Job as i32),
			"no job pool"
		);

		let servers_iter = job_server_rows
			.iter()
			.filter(|(dc_id, _)| dc_id == &datacenter_id);
		let server_count = servers_iter.clone().count() as u64;

		// Aggregate total available memory from all job servers
		let default_memory = *unwrap!(default_memory.get(&datacenter_id));
		let total_memory = servers_iter.fold(0, |acc, (_, memory)| {
			acc + memory.map(|x| x as u64).unwrap_or(default_memory)
		});

		// Aggregate memory usage
		let total_used_memory = topology.servers.iter().fold(0, |acc_usage, server| {
			acc_usage
				+ server
					.usage
					.as_ref()
					.map(|stats| stats.memory)
					.unwrap_or_default()
		});

		// Calculate new desired count
		let current_desired_count = job_pool.desired_count;
		let new_desired_count = algorithm(
			datacenter_id,
			current_desired_count,
			server_count,
			default_memory,
			total_used_memory,
			total_memory,
		);

		if new_desired_count != current_desired_count {
			tracing::info!(
				current=%current_desired_count, new=%new_desired_count,
				"scaling datacenter {}", datacenter_id
			);

			let new_pools = datacenter
				.pools
				.iter()
				.cloned()
				.map(|mut pool| {
					if pool.pool_type == backend::cluster::PoolType::Job as i32 {
						pool.desired_count = new_desired_count;
					}

					pool
				})
				.collect::<Vec<_>>();

			msg!([ctx] cluster::msg::datacenter_update(datacenter_id) {
				datacenter_id: datacenter.datacenter_id,
				pools: new_pools,
				drain_timeout: None,
			})
			.await?;
		}
	}

	Ok(())
}

fn algorithm(
	datacenter_id: Uuid,
	_current_desired_count: u32,
	server_count: u64,
	default_memory_per_server: u64,
	used_memory: u64,
	total_memory: u64,
) -> u32 {
	let total_memory = apply_inaccuracy(total_memory);

	// Calculate how much total memory we should have assuming the first hardware choice was always chosen
	let expected_total = apply_inaccuracy(server_count * default_memory_per_server);
	
	// Calculate by how much our previous prediction was off
	let error = util::div_up!(
		expected_total.saturating_sub(total_memory),
		default_memory_per_server
	);

	// Calculate average usage
	let usage = util::div_up!(used_memory, default_memory_per_server);

	tracing::info!(
		usage=%used_memory, total=%total_memory, %expected_total, %error,
		"calculating datacenter {}", datacenter_id
	);

	(*SERVER_PROVISION_MARGIN + error + usage) as u32
}

// Linode servers do not actually give you the advertised amount of memory, we account for this error here
fn apply_inaccuracy(x: u64) -> u64 {
	(x * 96) / 100
}
