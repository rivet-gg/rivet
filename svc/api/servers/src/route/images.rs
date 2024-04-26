use api_helper::ctx::Ctx;
use proto::backend::pkg::*;
use rivet_api::models;
use rivet_operation::prelude::*;
use serde_json::json;

use crate::auth::Auth;

// MARK: POST /images
pub async fn create(
	ctx: Ctx<Auth>,
	body: models::ServersImagesCreateRequest,
) -> GlobalResult<serde_json::Value> {
	todo!();

	Ok(json!({}))
}
