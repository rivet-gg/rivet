use proto::backend::pkg::*;
use rivet_operation::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NodeDrain {
	node: nomad_client::models::Node,
}

pub async fn handle(
	client: chirp_client::Client,
	NodeDrain { node }: &NodeDrain,
) -> GlobalResult<()> {
	let node_id = unwrap_ref!(node.ID);
	let meta = unwrap_ref!(node.meta, "no metadata on node");
	let server_id = util::uuid::parse(unwrap!(meta.get("server-id"), "no server-id in metadata"))?;

	tracing::info!(?server_id, ?node, "drain---------------");

	Ok(())
}
