use api_helper::anchor::WatchResponse;
use rivet_api::models;
use rivet_operation::prelude::*;

pub async fn database_for_namespace(
	ctx: &OperationContext<()>,
	namespace_id: common::Uuid,
) -> GlobalResult<()> {
	let namespaces_res = op!([ctx] game_namespace_get {
		namespace_ids: vec![namespace_id],
	})
	.await?;
	let namespace = internal_unwrap_owned!(namespaces_res.namespaces.first());

	todo!();

	// let version_id = internal_unwrap!(namespace.version_id);
	// let config_res = op!([ctx] database_game_version_get {
	// 	version_ids: vec![*version_id],
	// })
	// .await?;

	// if config_res.versions.first().is_none() {
	// 	panic_with!(
	// 		API_FORBIDDEN,
	// 		reason = "KV service not enabled for this namespace"
	// 	);
	// }

	Ok(())
}

pub fn watch_response(value: WatchResponse) -> models::WatchResponse {
	models::WatchResponse {
		index: value.watch_index().to_owned(),
	}
}
