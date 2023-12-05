use chirp_worker::prelude::*;
use proto::backend::pkg::*;
use nomad_client::{models, apis::{configuration::Configuration, nodes_api}};

// TODO: Remove once nomad-client is updated to the hashicorp openapi client everywhere in the codebase
pub fn config_from_env() -> GlobalResult<Configuration> {
	let nomad_url = unwrap!(
		std::env::var("NOMAD_URL").ok(),
		"no NOMAD_URL env var"
	);
	let config = Configuration {
		base_path: format!("{}/v1", nomad_url),
		..Default::default()
	};

	Ok(config)
}

lazy_static::lazy_static! {
	static ref NOMAD_CONFIG: Configuration =
		config_from_env().unwrap();
}

#[worker(name = "cluster-server-undrain")]
async fn worker(ctx: &OperationContext<cluster::msg::server_undrain::Message>) -> GlobalResult<()> {
	let server_id = unwrap!(ctx.server_id).as_uuid();
	
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
		retry_bail!("server does not have nomad running yet");
	};
	
	// TODO: Check for idempotence
	nodes_api::update_node_drain(
		&NOMAD_CONFIG,
		&nomad_node_id,
		models::NodeUpdateDrainRequest {
			drain_spec: Some(Box::new(models::DrainSpec {
				deadline: Some(todo!()),
				ignore_system_jobs: None,
			})),
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
