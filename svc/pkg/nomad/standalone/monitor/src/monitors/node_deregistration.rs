use proto::backend::pkg::*;
use rivet_operation::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NodeDeregistration {
	node: nomad_client::models::Node,
}

pub async fn handle(
	client: chirp_client::Client,
	NodeDeregistration { node }: &NodeDeregistration,
) -> GlobalResult<()> {
	let node_id = unwrap_ref!(node.ID);
	let meta = unwrap_ref!(node.meta, "no metadata on node");
	let cluster_id =
		util::uuid::parse(unwrap!(meta.get("cluster_id"), "no cluster_id in metadata"))?;
	let region_id = util::uuid::parse(unwrap!(meta.get("region_id"), "no region_id in metadata"))?;

	msg!([client] nomad::msg::monitor_node_deregistered(node_id) {
		cluster_id: Some(cluster_id.into()),
		region_id: Some(region_id.into()),
		node_id: node_id.to_owned(),
	})
	.await?;

	Ok(())
}
