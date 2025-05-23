use api_helper::define_router;
use hyper::{Body, Request, Response};
use rivet_operation::prelude::*;

pub async fn handle(
	shared_client: chirp_client::SharedClientHandle,
	config: rivet_config::Config,
	pools: rivet_pools::Pools,
	cache: rivet_cache::Cache,
	ray_id: uuid::Uuid,
	request: Request<Body>,
) -> Result<Response<Body>, http::Error> {
	let response = Response::builder();

	// Handle route
	Router::handle(
		shared_client,
		config,
		pools,
		cache,
		ray_id,
		request,
		response,
	)
	.await
}

define_router! {
	routes: {},
	mounts: [
		{
			path: api_ui::route::Router,
		},
		{
			path: api_auth::route::Router,
			prefix: "auth",
		},
		{
			path: api_cf_verification::route::Router,
			prefix: "cf-verification",
		},
		{
			path: api_cloud::route::Router,
			prefix: "cloud",
		},
		{
			path: api_games::route::Router,
		},
		{
			path: api_group::route::Router,
			prefix: "group",
		},
		{
			path: api_identity::route::Router,
			prefix: "identity",
		},
		{
			path: api_job::route::Router,
			prefix: "job",
		},
		{
			path: api_matchmaker::route::Router,
			prefix: "matchmaker",
		},
		{
			path: api_portal::route::Router,
			prefix: "portal",
		},
		{
			path: api_status::route::Router,
			prefix: "status",
		},
		{
			path: api_core_actor::route::Router,
		},
	],
}
