use rivet_operation::prelude::*;

pub mod route;

pub async fn start(config: rivet_config::Config, pools: rivet_pools::Pools) -> GlobalResult<()> {
	api_helper::start(
		config.clone(),
		pools,
		config.server()?.rivet.ports.api_internal,
		route::handle,
	)
	.await;
	Ok(())
}
