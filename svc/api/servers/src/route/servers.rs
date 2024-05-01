use api_helper::ctx::Ctx;
use rivet_api::models;
use rivet_convert::{ApiInto, ApiTryInto};
use rivet_operation::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::Auth;

// MARK: POST /servers
pub async fn create(
	ctx: Ctx<Auth>,
	body: models::ServersCreateServerRequest,
) -> GlobalResult<models::ServersCreateServerResponse> {
	let game_id = ctx.auth().server()?.game_id;
	let games = op!([ctx] cluster_get_for_game {
		game_ids: vec![game_id.into()]
	})
	.await?
	.games;

	let cluster_id = unwrap!(unwrap!(games.first()).cluster_id);

	let datacenters = op!([ctx] cluster_datacenter_resolve_for_name_id {
		cluster_id: Some(cluster_id),
		name_ids: vec![body.datacenter.clone()]
	})
	.await?
	.datacenters;

	if datacenters.is_empty() {
		bail_with!(CLUSTER_DATACENTER_NOT_FOUND);
	}

	let datacenter_id = unwrap!(unwrap!(datacenters.first()).datacenter_id);

	let metadata = serde_json::from_value(body.metadata.unwrap_or_default())?;

	let server = op!([ctx] ds_server_create {
		cluster_id: Some(cluster_id),
		datacenter_id: Some(datacenter_id),
		resources: Some((*body.resources).api_into()),
		kill_timeout_ms: body.kill_timeout.unwrap_or_default(),
		metadata: metadata,
		runtime: Some((*body.runtime).api_try_into()?),
	})
	.await?
	.server;

	Ok(models::ServersCreateServerResponse {
		server: Box::new(unwrap!(server).api_try_into()?),
	})
}

// MARK: DELETE /servers/{server_id}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteQuery {
	override_kill_timeout: Option<i64>,
}

pub async fn destroy(
	ctx: Ctx<Auth>,
	server_id: Uuid,
	query: DeleteQuery,
) -> GlobalResult<models::ServersDestroyServerResponse> {
	let server_id = op!([ctx] ds_server_delete {
		server_id: Some(server_id.into()),
		override_kill_timeout_ms: query.override_kill_timeout.unwrap_or_default(),
	})
	.await?
	.server_id;

	Ok(models::ServersDestroyServerResponse {
		server_id: unwrap!(server_id).as_uuid(),
	})
}
