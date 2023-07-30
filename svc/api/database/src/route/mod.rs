use api_helper::{define_router, util::CorsConfigBuilder};
use hyper::{Body, Request, Response};
use rivet_api::models;

mod collections;

pub async fn handle(
	shared_client: chirp_client::SharedClientHandle,
	pools: rivet_pools::Pools,
	cache: rivet_cache::Cache,
	ray_id: uuid::Uuid,
	request: Request<Body>,
) -> Result<Response<Body>, http::Error> {
	let response = Response::builder();

	// Handle route
	Router::handle(shared_client, pools, cache, ray_id, request, response).await
}

define_router! {
	cors: CorsConfigBuilder::public().build(),
	routes: {
		"collections" / String / "fetch": {
			POST: collections::fetch(body: models::DatabaseFetchRequest),
		},
		"collections" / String / "insert": {
			POST: collections::insert(body: models::DatabaseInsertRequest),
		},
		"collections" / String / "update": {
			POST: collections::update(body: models::DatabaseUpdateRequest),
		},
		"collections" / String / "delete": {
			POST: collections::delete(body: models::DatabaseDeleteRequest),
		},
	},
}
