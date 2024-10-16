use rivet_operation::prelude::*;

pub mod route;

pub async fn start(config: rivet_config::Config, pools: rivet_pools::Pools) -> GlobalResult<()> {
	api_helper::start(config, pools, route::handle).await;
	Ok(())
}
