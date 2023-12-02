use std::collections::HashMap;

use nomad_client::apis::{allocations_api, configuration::Configuration, nodes_api};
use proto::backend::pkg::*;
use rivet_operation::prelude::*;

// TODO: Remove once nomad-client is updated to the hashicorp openapi client everywhere in the codebase
pub fn config_from_env() -> Result<Configuration, nomad_util::NomadError> {
	let nomad_url = std::env::var("NOMAD_URL")
		.map_err(|_| nomad_util::NomadError::MissingEnvVar("NOMAD_URL".into()))?;
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

#[derive(sqlx::FromRow)]
struct Server {
	cluster_id: Uuid,
	nomad_node_id: String,
}

#[derive(Default)]
struct Stats {
	cpu: i64,
	memory: i64,
	disk: i64,
}

#[operation(name = "cluster-topology-get")]
pub async fn handle(
	ctx: OperationContext<cluster::topology_get::Request>,
) -> GlobalResult<cluster::topology_get::Response> {
	let cluster_ids = ctx
		.cluster_ids
		.iter()
		.map(common::Uuid::as_uuid)
		.collect::<Vec<_>>();

	let servers = sql_fetch_all!(
		[ctx, Server]
		"
		SELECT
			cluster_id,
			nomad_node_id
		FROM db_cluster_state.servers
		WHERE cluster_id = ANY($1)
		",
		cluster_ids
	)
	.await?;

	let mut clusters = HashMap::new();

	// Fetch batch data from nomad
	let (allocation_info, node_info) = tokio::try_join!(
		async {
			allocations_api::get_allocations(
				&NOMAD_CONFIG,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				Some(true),
				None,
			)
			.await
			.map_err(Into::<GlobalError>::into)
		},
		async {
			nodes_api::get_nodes(
				&NOMAD_CONFIG,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				Some(true),
			)
			.await
			.map_err(Into::<GlobalError>::into)
		},
	)?;

	for server in servers {
		let mut usage = Stats::default();

		// Aggregate all allocated resources for this node
		for alloc in &allocation_info {
			let alloc_node_id = unwrap_ref!(alloc.node_id);

			if alloc_node_id == &server.nomad_node_id {
				let resources = unwrap_ref!(alloc.allocated_resources);
				let shared_resources = unwrap_ref!(resources.shared);
				let tasks = unwrap_ref!(resources.tasks);

				for task in tasks.values() {
					let cpu = unwrap_ref!(task.cpu);
					let memory = unwrap_ref!(task.memory);

					usage.cpu += unwrap!(cpu.cpu_shares);
					usage.memory += unwrap!(memory.memory_mb);
				}

				usage.disk += unwrap!(shared_resources.disk_mb);
			}
		}

		// Get total node resources
		let node = unwrap!(
			node_info.iter().find(|node| node
				.ID
				.as_ref()
				.map_or(false, |node_id| node_id == &server.nomad_node_id)),
			"node not found"
		);
		let resources = unwrap_ref!(node.node_resources);
		let total = Stats {
			cpu: unwrap!(unwrap_ref!(resources.cpu).cpu_shares),
			memory: unwrap!(unwrap_ref!(resources.memory).memory_mb),
			disk: unwrap!(unwrap_ref!(resources.disk).disk_mb),
		};

		let cluster = clusters.entry(server.cluster_id).or_insert_with(|| {
			cluster::topology_get::response::Cluster {
				cluster_id: Some(server.cluster_id.into()),
				topologies: Vec::new(),
			}
		});

		cluster
			.topologies
			.push(cluster::topology_get::response::Topology {
				node_id: server.nomad_node_id,
				cpu: usage.cpu as f32 / total.cpu as f32,
				memory: usage.memory as f32 / total.memory as f32,
				disk: usage.disk as f32 / total.disk as f32,
			});
	}

	Ok(cluster::topology_get::Response {
		clusters: clusters.into_values().collect::<Vec<_>>(),
	})
}
