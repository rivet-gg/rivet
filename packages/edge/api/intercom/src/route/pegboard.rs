use api_helper::ctx::Ctx;
use rivet_api::models;
use rivet_operation::prelude::*;
use serde_json::json;

use crate::auth::Auth;

// MARK: POST /pegboard/image/{}/prewarm
pub async fn prewarm_image(
	ctx: Ctx<Auth>,
	image_id: Uuid,
	body: models::EdgeIntercomPegboardPrewarmImageRequest,
) -> GlobalResult<serde_json::Value> {
	ctx.auth().server(&ctx)?;

	let client_id: Uuid = todo!("choose client");
	ctx.signal(edge_pegboard::workflows::client::PrewarmImage {
		image_id,
		image_artifact_url_stub: body.image_artifact_url_stub,
	})
	.to_workflow::<edge_pegboard::workflows::client::Workflow>()
	.tag("client_id", client_id)
	.send()
	.await?;

	Ok(json!({}))
}

// MARK: POST /pegboard/client/{}/toggle-drain
pub async fn drain_client(
	ctx: Ctx<Auth>,
	client_id: Uuid,
	body: models::EdgeIntercomPegboardToggleClientDrainRequest,
) -> GlobalResult<serde_json::Value> {
	ctx.auth().server(&ctx)?;

	todo!("publish drain/undrain signal to client wf");

	Ok(json!({}))
}
