use std::{collections::HashMap, iter::Iterator};

use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

lazy_static::lazy_static! {
	static ref JOB_SERVER_PROVISION_MARGIN: u64 = util::env::var("JOB_SERVER_PROVISION_MARGIN").unwrap()
		.parse()
		.unwrap();
}

#[derive(sqlx::FromRow)]
struct Server {
	server_id: Uuid,
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

	// TODO: Remove
	// let servers = sql_fetch_all!(
	// 	[ctx, (Uuid,), &crdb]
	// 	"
	// 	SELECT server_id
	// 	FROM db_cluster.linode_misc
	// 	",
	// )
	// .await?;

	// for (server_id,) in servers {
	// 	msg!([ctx] cluster::msg::server_destroy(server_id) {
	// 		server_id: Some(server_id.into()),
	// 		force: true,
	// 	})
	// 	.await?;
	// }

	// return Ok(());

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
			SELECT server_id, datacenter_id, pool_type, memory
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

		let job_pool = unwrap!(
			datacenter
				.pools
				.iter()
				.find(|pool| pool.pool_type == backend::cluster::PoolType::Job as i32),
			"no job pool"
		);
		let gg_pool = unwrap!(
			datacenter
				.pools
				.iter()
				.find(|pool| pool.pool_type == backend::cluster::PoolType::Gg as i32),
			"no gg pool"
		);

		let default_memory = *unwrap!(default_memory.get(&datacenter_id));

		let servers_in_datacenter = servers
			.iter()
			.filter(|server| server.datacenter_id == datacenter_id);

		// Calculate new desired counts
		let new_job_desired_count = autoscale_job_servers(
			default_memory,
			servers_in_datacenter.clone(),
			topology,
		)
		.await?;
		let new_gg_desired_count =
			autoscale_gg_servers(servers_in_datacenter).await?;

		let new_job_desired_count = 1;
		let new_gg_desired_count = 1;

		if new_job_desired_count != job_pool.desired_count
			|| new_gg_desired_count != gg_pool.desired_count
		{
			tracing::info!(
				%new_job_desired_count, %new_gg_desired_count,
				"scaling datacenter {}", datacenter_id
			);

			let mut new_job_pool = job_pool.clone();
			new_job_pool.desired_count = new_job_desired_count;

			let mut new_gg_pool = gg_pool.clone();
			new_gg_pool.desired_count = new_gg_desired_count;

			msg!([ctx] cluster::msg::datacenter_update(datacenter_id) {
				datacenter_id: datacenter.datacenter_id,
				pools: vec![new_job_pool],
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
		servers.filter(|server| server.pool_type == backend::cluster::PoolType::Job as i32 as i64);
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
		usage=%used_memory, total=%total_memory, %expected_total, %error,
		"calculating job server count"
	);

	(*JOB_SERVER_PROVISION_MARGIN + error + usage) as u32
}

async fn autoscale_gg_servers<'a, I: Iterator<Item = &'a Server>>(
	servers: I,
) -> GlobalResult<u32> {
	let gg_servers_iter =
		servers.filter(|server| server.pool_type == backend::cluster::PoolType::Gg as i32 as i64);

	"last_over_time(nomad_client_allocs_memory_allocated{{exported_job=\"{nomad_job_id}\",task=\"{task}\"}} [15m:15s]) or vector(0)"
	
	handle_request(
		&PROMETHEUS_URL,
		None,
		formatdoc!(
			r#"
			sum without (mode) (
				irate(
					node_cpu_seconds_total{
						datacenter_id="{datacenter_id}",
						pool_type="gg",

						mode!="idle",
						mode!="iowait",
						mode!="steal"
					}
					[5m]
				)
			) * 100
			"#
		),
		nomad_job_id = metric.job,
		task = metric.task
	)).await?;

	let new_desired_count = gg_autoscale_algorithm();

	Ok(new_desired_count)
}

fn gg_autoscale_algorithm(current_desired_count: u32, server_count: u64, used_cpu: u64) -> u32 {
	let total_cpu = server_count * 100;
	let diff = total_cpu.saturating_sub(used_cpu);

	if diff < 20 {
		current_desired_count + 1
	} else if diff > 130 {
		current_desired_count - 1
	}
	else {
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
	values: Option<Vec<(u64, f64)>>,
}

#[derive(Debug)]
struct QueryTiming {
	start: i64,
	end: i64,
	step: i64,
}

impl QueryTiming {
	fn new(start: i64, end: i64, step: i64) -> Self {
		QueryTiming { start, end, step }
	}
}

lazy_static::lazy_static! {
	static ref PROMETHEUS_URL: String = util::env::var("PROMETHEUS_URL").unwrap();
}

// TODO: Copied from job-run-metrics-log
async fn handle_request(
	url: &String,
	timing: Option<&QueryTiming>,
	query: String,
) -> GlobalResult<PrometheusResult> {
	// Start query string building
	let mut query_pairs = vec![("query", query), ("timeout", "2500ms".to_owned())];

	// Append timing queries
	if let Some(timing) = timing {
		query_pairs.push(("start", (timing.start / 1000).to_string()));
		query_pairs.push(("end", (timing.end / 1000).to_string()));
		query_pairs.push(("step", format!("{}ms", timing.step)));
	}

	let query_string = serde_urlencoded::to_string(query_pairs)?;
	let req_url = format!(
		"{}/api/v1/query{}?{}",
		url,
		if timing.is_some() { "_range" } else { "" },
		query_string
	);
	tracing::info!(?req_url, "prometheus query");

	// Query prometheus
	let res = reqwest::Client::new().get(req_url).send().await?;

	if res.status() != StatusCode::OK {
		let status = res.status();
		let text = res.text().await?;

		return Err(Error::PrometheusError(format!(
			"failed prometheus request: ({}) {}",
			status, text
		))
		.into());
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
