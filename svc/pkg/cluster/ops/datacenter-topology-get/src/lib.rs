use std::collections::HashMap;

use nomad_client::apis::{allocations_api, configuration::Configuration, nodes_api};
use proto::backend::pkg::*;
use rivet_operation::prelude::*;

lazy_static::lazy_static! {
	static ref NOMAD_CONFIG: Configuration =
	nomad_util::new_config_from_env().unwrap();
}

#[derive(sqlx::FromRow)]
struct Server {
	server_id: Uuid,
	datacenter_id: Uuid,
	nomad_node_id: String,
}

#[derive(Default)]
struct Stats {
	cpu: i64,
	memory: i64,
	disk: i64,
}

#[operation(name = "cluster-datacenter-topology-get")]
pub async fn handle(
	ctx: OperationContext<cluster::datacenter_topology_get::Request>,
) -> GlobalResult<cluster::datacenter_topology_get::Response> {
	let datacenter_ids = ctx
		.datacenter_ids
		.iter()
		.map(common::Uuid::as_uuid)
		.collect::<Vec<_>>();

	let servers = sql_fetch_all!(
		[ctx, Server]
		"
		SELECT
			server_id, datacenter_id, nomad_node_id
		FROM db_cluster.servers
		WHERE
			datacenter_id = ANY($1) AND
			nomad_node_id IS NOT NULL
		",
		&datacenter_ids,
	)
	.await?;

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

	// Fill in empty datacenters
	let mut datacenters = datacenter_ids
		.iter()
		.map(|datacenter_id| {
			(
				*datacenter_id,
				cluster::datacenter_topology_get::response::Datacenter {
					datacenter_id: Some((*datacenter_id).into()),
					servers: Vec::new(),
				},
			)
		})
		.collect::<HashMap<_, _>>();

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

		let datacenter = datacenters.entry(server.datacenter_id).or_insert_with(|| {
			cluster::datacenter_topology_get::response::Datacenter {
				datacenter_id: Some(server.datacenter_id.into()),
				servers: Vec::new(),
			}
		});

		datacenter
			.servers
			.push(cluster::datacenter_topology_get::response::Server {
				server_id: Some(server.server_id.into()),
				node_id: server.nomad_node_id,
				cpu: usage.cpu as f32 / total.cpu as f32,
				memory: usage.memory as f32 / total.memory as f32,
				disk: usage.disk as f32 / total.disk as f32,
			});
	}

	Ok(cluster::datacenter_topology_get::Response {
		datacenters: datacenters.into_values().collect::<Vec<_>>(),
	})
}
