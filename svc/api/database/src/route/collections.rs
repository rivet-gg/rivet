use api_helper::{
	anchor::{WatchIndexQuery, WatchResponse},
	ctx::Ctx,
};
use chirp_client::TailAnchorResponse;
use proto::backend::pkg::*;
use rivet_api::models;
use rivet_operation::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{auth::Auth, utils};

#[derive(Deserialize, Serialize)]
pub struct SingleQuery {
	key: String,
	namespace_id: Option<Uuid>,
}

// MARK: POST /fetch
pub async fn fetch(
	ctx: Ctx<Auth>,
	collection: String,
	body: models::DatabaseFetchRequest,
) -> GlobalResult<models::DatabaseFetchResponse> {
	let database_id = ctx.auth().database(ctx.op_ctx(), body.database_id).await?;

	todo!()

	// TODO:

	// Ok(models::KvFetchResponse {
	// 	entries: TODO,
	// })
}

// MARK: POST /insert
pub async fn insert(
	ctx: Ctx<Auth>,
	collection: String,
	body: models::DatabaseInsertRequest,
) -> GlobalResult<models::DatabaseInsertResponse> {
	Ok(todo!())
}

// MARK: POST /update
pub async fn update(
	ctx: Ctx<Auth>,
	collection: String,
	body: models::DatabaseUpdateRequest,
) -> GlobalResult<models::DatabaseUpdateResponse> {
	Ok(todo!())
}

// MARK: POST /dlete
pub async fn delete(
	ctx: Ctx<Auth>,
	collection: String,
	body: models::DatabaseDeleteRequest,
) -> GlobalResult<models::DatabaseDeleteResponse> {
	Ok(todo!())
}
