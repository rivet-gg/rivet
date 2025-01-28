use api_helper::{
	anchor::{WatchIndexQuery, WatchResponse},
	ctx::Ctx,
};
use rivet_api::models;
use rivet_operation::prelude::*;

use crate::auth::Auth;

// MARK: GET /events/live
pub async fn events(
	ctx: Ctx<Auth>,
	_watch_index: WatchIndexQuery,
) -> GlobalResult<models::IdentityWatchEventsResponse> {
	Ok(models::IdentityWatchEventsResponse {
		events: Vec::new(),
		watch: WatchResponse::new_as_model(util::timestamp::now()),
	})
}
