use std::{collections::HashMap, iter::Iterator};

use indoc::formatdoc;
use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
use serde::Deserialize;

lazy_static::lazy_static! {
	static ref JOB_SERVER_PROVISION_MARGIN: u64 = util::env::var("JOB_SERVER_PROVISION_MARGIN").unwrap()
		.parse()
		.unwrap();
}

#[derive(sqlx::FromRow)]
struct Server {
	datacenter_id: Uuid,
	pool_type: i64,
	memory: Option<i64>,
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
	let crdb = ctx.crdb().await?;

	// Fetch all datacenters and all of their gg + job servers
	let (datacenter_rows, servers) = tokio::try_join!(
		sql_fetch_all!(
			[ctx, (Uuid,), &crdb]
			"
			SELECT datacenter_id
			FROM db_cluster.datacenters
			",
		),
		sql_fetch_all!(
			[ctx, Server, &crdb]
			"
			SELECT datacenter_id, pool_type, memory
			FROM db_cluster.servers
			WHERE
				pool_type = ANY($1) AND
				cloud_destroy_ts IS NULL AND
				taint_ts IS NULL
			",
			&[
				backend::cluster::PoolType::Job as i64,
				backend::cluster::PoolType::Gg as i64
			]
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
		// Gracefully fetch job pool
		.filter_map(|dc| {
			let job_pool = dc
				.pools
				.iter()
				.find(|pool| pool.pool_type == backend::cluster::PoolType::Job as i32);

			if let Some(job_pool) = job_pool {
				Some((dc, job_pool))
			} else {
				tracing::warn!(datacenter_id=?dc.datacenter_id, "datacenter has no job pool");

				None
			}
		})
		.map(|(dc, job_pool)| {
			let datacenter_id = unwrap_ref!(dc.datacenter_id).as_uuid();
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

	// Convert the memory data into a hashmap for better reads
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

	// Autoscale each datacenter
	for datacenter in &datacenters_res.datacenters {
		let datacenter_id = unwrap_ref!(datacenter.datacenter_id).as_uuid();
		let topology = unwrap!(topologies_res
			.datacenters
			.iter()
			.find(|topo| topo.datacenter_id == datacenter.datacenter_id));

		let job_pool = datacenter
			.pools
			.iter()
			.find(|pool| pool.pool_type == backend::cluster::PoolType::Job as i32);
		let gg_pool = datacenter
			.pools
			.iter()
			.find(|pool| pool.pool_type == backend::cluster::PoolType::Gg as i32);

		// Gracefully handle missing pools (test datacenters might not have them)
		let (Some(job_pool), Some(gg_pool)) = (job_pool, gg_pool) else {
			tracing::warn!(?datacenter_id, "datacenter missing job/gg pools");
			continue;
		};

		let default_memory = *unwrap!(default_memory.get(&datacenter_id));

		let servers_in_datacenter = servers
			.iter()
			.filter(|server| server.datacenter_id == datacenter_id);

		// Calculate new desired counts
		let new_job_desired_count =
			autoscale_job_servers(default_memory, servers_in_datacenter.clone(), topology).await?;
		let new_gg_desired_count =
			autoscale_gg_servers(datacenter_id, gg_pool.desired_count).await?;

		if new_job_desired_count != job_pool.desired_count
			|| new_gg_desired_count != gg_pool.desired_count
		{
			tracing::info!(
				old_job=%job_pool.desired_count,
				new_job=%new_job_desired_count,
				old_gg=%gg_pool.desired_count,
				new_gg=%new_gg_desired_count,
				"scaling datacenter {}", datacenter_id
			);

			msg!([ctx] cluster::msg::datacenter_update(datacenter_id) {
				datacenter_id: datacenter.datacenter_id,
				pools: vec![
					cluster::msg::datacenter_update::PoolUpdate {
						pool_type: backend::cluster::PoolType::Job as i32,
						hardware: Vec::new(),
						desired_count: Some(new_job_desired_count),
						max_count: None,
					},
					cluster::msg::datacenter_update::PoolUpdate {
						pool_type: backend::cluster::PoolType::Gg as i32,
						hardware: Vec::new(),
						desired_count: Some(new_gg_desired_count),
						max_count: None,
					}
				],
				drain_timeout: None,
			})
			.await?;
		}
	}

	Ok(())
}

async fn autoscale_job_servers<'a, I: Iterator<Item = &'a Server> + Clone>(
	default_memory: u64,
	servers: I,
	topology: &cluster::datacenter_topology_get::response::Datacenter,
) -> GlobalResult<u32> {
	let job_servers_iter =
		servers.filter(|server| server.pool_type == backend::cluster::PoolType::Job as i64);
	let server_count = job_servers_iter.clone().count() as u64;

	// Aggregate total available memory from all job servers. We assume a server has this default memory
	// amount (memory of the first hardware in the list) before it is provisioned
	let total_memory = job_servers_iter.fold(0, |acc, server| {
		acc + server.memory.map(|x| x as u64).unwrap_or(default_memory)
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

	let new_desired_count = job_autoscale_algorithm(
		server_count,
		default_memory,
		total_used_memory,
		total_memory,
	);

	Ok(new_desired_count)
}

fn job_autoscale_algorithm(
	server_count: u64,
	default_memory_per_server: u64,
	used_memory: u64,
	total_memory: u64,
) -> u32 {
	// Calculate how much total memory we should have assuming the first hardware choice was always chosen
	let expected_total = server_count * default_memory_per_server;

	// Calculate by how much our previous prediction was off
	let error = apply_inaccuracy(expected_total.saturating_sub(total_memory));
	let error = util::div_up!(error, default_memory_per_server);

	// Calculate average usage
	let usage = util::div_up!(used_memory, default_memory_per_server);

	tracing::info!(
		%used_memory, %total_memory, expected_total_memory=%expected_total, %error,
		"calculating job server count"
	);

	(*JOB_SERVER_PROVISION_MARGIN + error + usage) as u32
}

async fn autoscale_gg_servers(
	datacenter_id: Uuid,
	current_desired_count: u32,
) -> GlobalResult<u32> {
	let prom_res = handle_request(
		&PROMETHEUS_URL,
		formatdoc!(
			r#"
			last_over_time((
				sum without (mode) (
					irate(
						node_cpu_seconds_total{{
							datacenter_id="{datacenter_id}",
							pool_type="gg",

							mode!="idle",
							mode!="iowait",
							mode!="steal"
						}}
						[5m]
					)
				) * 100
			) [15m:15s])
			or vector(0)
			"#
		),
	)
	.await?;
	let (_, cpu_sum) = unwrap!(prom_res.value);
	#[allow(clippy::cast_possible_truncation)]
	let cpu_sum = cpu_sum.parse::<f64>()? as u64;

	let new_desired_count = gg_autoscale_algorithm(current_desired_count, cpu_sum);

	Ok(new_desired_count)
}

fn gg_autoscale_algorithm(current_desired_count: u32, used_cpu: u64) -> u32 {
	let total_cpu = current_desired_count as u64 * 100;
	let diff = total_cpu.saturating_sub(used_cpu);

	tracing::info!(
		%used_cpu, %total_cpu, %diff,
		"calculating gg server count"
	);

	if diff < 20 {
		current_desired_count + 1
	} else if diff > 130 {
		current_desired_count - 1
	} else {
		current_desired_count
	}
}

#[derive(Debug, Deserialize)]
struct PrometheusResponse {
	data: PrometheusData,
}

#[derive(Debug, Deserialize)]
struct PrometheusData {
	#[serde(rename = "resultType")]
	_result_type: String,
	result: Vec<PrometheusResult>,
}

#[derive(Debug, Clone, Deserialize)]
struct PrometheusResult {
	value: Option<(f64, String)>,
}

lazy_static::lazy_static! {
	static ref PROMETHEUS_URL: String = util::env::var("PROMETHEUS_URL").unwrap();
}

// TODO: Copied from job-run-metrics-log
async fn handle_request(url: &String, query: String) -> GlobalResult<PrometheusResult> {
	let query_pairs = vec![("query", query), ("timeout", "2500ms".to_owned())];

	let query_string = serde_urlencoded::to_string(query_pairs)?;
	let req_url = format!("{}/api/v1/query?{}", url, query_string);

	// Query prometheus
	let res = reqwest::Client::new().get(req_url).send().await?;

	if !res.status().is_success() {
		let status = res.status();
		let text = res.text().await?;

		bail!(format!("failed prometheus request: ({}) {}", status, text));
	}

	let body = res.json::<PrometheusResponse>().await?;
	let data = unwrap!(body.data.result.first()).clone();

	Ok(data)
}

// Linode servers do not actually give you the advertised amount of memory, we account for this error here
// https://www.linode.com/community/questions/17791/why-doesnt-free-m-match-the-full-amount-of-ram-of-my-nanode-plan
fn apply_inaccuracy(x: u64) -> u64 {
	(x * 96) / 100
}
