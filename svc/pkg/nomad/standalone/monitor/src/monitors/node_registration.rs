use proto::backend::pkg::*;
use rivet_operation::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NodeRegistration {
	node: nomad_client::models::Node,
}

#[tracing::instrument(skip_all)]
pub async fn start(
	shared_client: chirp_client::SharedClientHandle,
	redis_job: RedisPool,
) -> GlobalResult<()> {
	let redis_index_key = "nomad:monitor_index:node_registration_monitor";

	let configuration = nomad_util::config_from_env().unwrap();
	nomad_util::monitor::Monitor::run(
		configuration,
		redis_job,
		redis_index_key,
		&["Node"],
		move |event| {
			let client = shared_client
				.clone()
				.wrap_new("nomad-node-registration-monitor");
			async move {
				if let Some(payload) = event
					.decode::<NodeRegistration>("Node", "NodeRegistration")
					.unwrap()
				{
					let spawn_res = tokio::task::Builder::new()
						.name("nomad_node_registration_monitor::handle_event")
						.spawn(handle(client, payload));
					if let Err(err) = spawn_res {
						tracing::error!(?err, "failed to spawn handle_event task");
					}
				}
			}
		},
	)
	.await?;

	Ok(())
}

#[tracing::instrument(skip_all)]
async fn handle(client: chirp_client::Client, payload: NodeRegistration) {
	match handle_inner(client, &payload).await {
		Ok(_) => {}
		Err(err) => {
			tracing::error!(?err, ?payload, "error handling event");
		}
	}
}

async fn handle_inner(
	client: chirp_client::Client,
	NodeRegistration { node }: &NodeRegistration,
) -> GlobalResult<()> {
	let node_id = unwrap_ref!(node.ID);
	let meta = unwrap_ref!(node.meta, "no metadata on node");
	let cluster_id =
		util::uuid::parse(unwrap!(meta.get("cluster_id"), "no cluster_id in metadata"))?;

	msg!([client] nomad::msg::monitor_node_registered(node_id) {
		cluster_id: Some(cluster_id.into()),
		node_id: node_id.to_owned(),
	})
	.await?;

	Ok(())
}
