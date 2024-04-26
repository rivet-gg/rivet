use api_helper::ctx::Ctx;
use proto::backend::pkg::*;
use rivet_api::models;
use rivet_operation::prelude::*;
use serde_json::json;

use crate::auth::Auth;

// MARK: POST /servers
pub async fn create(
	ctx: Ctx<Auth>,
	body: models::ServersServersCreateRequest,
) -> GlobalResult<models::ServersServersCreateResponse> {
	let clusters = op!([ctx] cluster_resolve_for_name_id {
		name_ids: vec![body.cluster.clone()]
	})
	.await?
	.clusters;

	if clusters.is_empty() {
		bail_with!(CLUSTER_NOT_FOUND);
	}

	let cluster_id = unwrap!(unwrap!(clusters.first()).cluster_id);

	let clusters = op!([ctx] cluster_get {
		cluster_ids: vec![cluster_id]
	})
	.await?
	.clusters;

	let cluster = match clusters.first() {
		Some(c) => c,
		None => bail_with!(CLUSTER_NOT_FOUND),
	};

	let datacenters = op!([ctx] cluster_datacenter_resolve_for_name_id {
		name_ids: vec![body.datacenter.clone()]
	})
	.await?
	.datacenters;

	if datacenters.is_empty() {
		bail_with!(CLUSTER_DATACENTER_NOT_FOUND);
	}

	let datacenter_id = unwrap!(unwrap!(datacenters.first()).datacenter_id);

	let datacenters = op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![datacenter_id]
	})
	.await?
	.datacenters;

	let datacenter = match datacenters.first() {
		Some(d) => d,
		None => bail_with!(CLUSTER_DATACENTER_NOT_FOUND),
	};

	Ok(models::ServersServersCreateResponse { server: todo!() })
}

// MARK: DELETE /servers/{server_id}
pub async fn delete(ctx: Ctx<Auth>, server_id: Uuid) -> GlobalResult<serde_json::Value> {
	todo!();

	Ok(json!({}))
}
