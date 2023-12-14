use chirp_worker::prelude::*;
use proto::backend::pkg::*;
use nomad_client::{models, apis::{configuration::Configuration, nodes_api}};

lazy_static::lazy_static! {
	static ref NOMAD_CONFIG: Configuration =
		nomad_util::new_config_from_env().unwrap();
}

#[worker(name = "cluster-server-undrain")]
async fn worker(ctx: &OperationContext<cluster::msg::server_undrain::Message>) -> GlobalResult<()> {
	let server_id = unwrap_ref!(ctx.server_id).as_uuid();

	// NOTE: `drain_ts` will already be set to null before this worker is called
	let (nomad_node_id,) = sql_fetch_one!(
		[ctx, (Option<String>,)]
		"
		SELECT
			nomad_node_id
		FROM db_cluster.servers
		WHERE server_id = $1
		",
		server_id
	)
	.await?;
	let Some(nomad_node_id) = nomad_node_id else {
		tracing::error!("server does not have nomad running yet, cannot undrain");
		return Ok(());
	};

	nodes_api::update_node_drain(
		&NOMAD_CONFIG,
		&nomad_node_id,
		models::NodeUpdateDrainRequest {
			drain_spec: None,
			mark_eligible: Some(true),
			meta: None,
			node_id: Some(nomad_node_id.clone()),
		},
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
	)
	.await?;

	Ok(())
}
