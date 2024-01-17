use rivet_operation::prelude::*;

fn main() -> GlobalResult<()> {
	rivet_runtime::run(start()).unwrap()
}

async fn start() -> GlobalResult<()> {
	let pools = rivet_pools::from_env("nomad-monitor").await?;
	let shared_client = chirp_client::SharedClient::from_env(pools.clone())?;
	let redis_job = pools.redis("persistent")?;

	tokio::task::Builder::new()
		.name("nomad_monitor::health_checks")
		.spawn(rivet_health_checks::run_standalone(
			rivet_health_checks::Config {
				pools: Some(pools.clone()),
			},
		))?;

	tokio::task::Builder::new()
		.name("nomad_monitor::metrics")
		.spawn(rivet_metrics::run_standalone())?;

	tokio::try_join!(
		nomad_monitor::monitors::alloc_plan::start(shared_client.clone(), redis_job.clone()),
		nomad_monitor::monitors::alloc_update::start(shared_client.clone(), redis_job.clone()),
		nomad_monitor::monitors::eval_update::start(shared_client.clone(), redis_job.clone()),
		nomad_monitor::monitors::node_registration::start(shared_client.clone(), redis_job.clone()),
	)?;

	Ok(())
}
