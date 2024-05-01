use futures_util::FutureExt;
use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

#[operation(name = "ds-server-create")]
pub async fn handle(
	ctx: OperationContext<dynamic_servers::server_create::Request>,
) -> GlobalResult<dynamic_servers::server_create::Response> {
	let resources = unwrap_ref!(ctx.resources).clone();
	let runtime = unwrap!(ctx.runtime.clone());

	let server_id = Uuid::new_v4();
	let game_id = unwrap_ref!(ctx.game_id).as_uuid();
	let cluster_id = unwrap_ref!(ctx.cluster_id).as_uuid();
	let datacenter_id = unwrap_ref!(ctx.datacenter_id).as_uuid();

	let create_ts = ctx.ts();

	rivet_pools::utils::crdb::tx(&ctx.crdb().await?, |tx| {
		let ctx = ctx.clone();
		let runtime = runtime.clone();

		async move {
			match runtime {
				dynamic_servers::server_create::request::Runtime::DockerRuntime(docker_runtime) => {
					#[derive(Default)]
					struct GameGuardUnnest {
						port_names: Vec<String>,
						port_numbers: Vec<Option<i32>>,
						protocols: Vec<i32>,
					}

					#[derive(Default)]
					struct HostUnnest {
						port_names: Vec<String>,
						port_numbers: Vec<Option<i32>>,
					}

					let docker_runtime_clone = docker_runtime.clone();

					let docker_network = unwrap!(docker_runtime.network);
					let (game_guard_unnest, host_unnest) =
						unwrap!(docker_network.ports.iter().try_fold(
							(GameGuardUnnest::default(), HostUnnest::default()),
							|(mut game_guard_unnest, mut host_unnest), (name, port)| {
								let routing = unwrap!(port.routing.clone());
								match routing {
									backend::dynamic_servers::docker_port::Routing::GameGuard(
										gameguard_protocol,
									) => {
										game_guard_unnest.port_names.push(name.clone());
										game_guard_unnest.port_numbers.push(port.port);
										game_guard_unnest
											.protocols
											.push(gameguard_protocol.protocol);
									}
									backend::dynamic_servers::docker_port::Routing::Host(_) => {
										host_unnest.port_names.push(name.clone());
										host_unnest.port_numbers.push(port.port);
									}
								};
								Ok::<_, GlobalError>((game_guard_unnest, host_unnest))
							},
						));

					sql_execute!(
						[ctx, @tx tx]
						"
						WITH
							servers_cte AS (
								INSERT INTO
									db_dynamic_servers.servers (
										server_id,
										game_id,
										datacenter_id,
										cluster_id,
										metadata,
										resources_cpu_millicores,
										resources_memory_mib,
										kill_timeout_ms,
										create_ts
									)
								VALUES
									($1, $2, $3, $4, $5, $6, $7, $8, $9)
								RETURNING
									1
							),
							docker_runtimes_cte AS (
								INSERT INTO
									db_dynamic_servers.docker_runtimes (
										server_id,
										image_id,
										args,
										network_mode,
										environment
									)
								VALUES
									($1, $10, $11, $12, $13)
								RETURNING
									1
							),
							docker_ports_host_cte AS (
								INSERT INTO
									db_dynamic_servers.docker_ports_host (
										server_id,
										port_name,
										port_number
									)
								SELECT
									$1,
									t.*
								FROM
									unnest($14, $15) AS t (port_name, port_number)
								RETURNING
									1
							),
							docker_ports_protocol_game_guard_cte AS (
								INSERT INTO
									db_dynamic_servers.docker_ports_protocol_game_guard (
										server_id,
										port_name,
										port_number,
										protocol
									)
								SELECT
									$1,
									t.*
								FROM
									unnest($16, $17, $18) AS t (port_name, port_number, protocol)
								RETURNING
									1
							)
						SELECT
							1
						",
						server_id,
						game_id,
						datacenter_id,
						cluster_id,
						serde_json::value::to_raw_value(&ctx.metadata.to_owned())?.to_string(), // 5
						resources.cpu_millicores,
						resources.memory_mib,
						ctx.kill_timeout_ms,
						create_ts,
						unwrap!(docker_runtime_clone.image_id).as_uuid(), // 10
						docker_runtime_clone.args,
						unwrap!(docker_runtime_clone.network).mode,
						serde_json::value::to_raw_value(&docker_runtime_clone.environment)?.to_string(),
						host_unnest.port_names,
						host_unnest.port_numbers, // 15
						game_guard_unnest.port_names,
						game_guard_unnest.port_numbers,
						game_guard_unnest.protocols,
					)
					.await?;
				}
			}

			Ok(())
		}
		.boxed()
	})
	.await?;

	Ok(dynamic_servers::server_create::Response {
		server: Some(backend::dynamic_servers::Server {
			server_id: Some(server_id.into()),
			game_id: Some(game_id.into()),
			datacenter_id: Some(datacenter_id.into()),
			cluster_id: Some(cluster_id.into()),
			metadata: ctx.metadata.clone(),
			resources: Some(backend::dynamic_servers::ServerResources {
				cpu_millicores: resources.cpu_millicores,
				memory_mib: resources.memory_mib,
			}),
			kill_timeout_ms: ctx.kill_timeout_ms,
			create_ts,
			destroy_ts: None,
			runtime: Some(match runtime {
				dynamic_servers::server_create::request::Runtime::DockerRuntime(docker_runtime) => {
					backend::dynamic_servers::server::Runtime::DockerRuntime(
						backend::dynamic_servers::DockerRuntime {
							args: docker_runtime.args,
							environment: docker_runtime.environment,
							image_id: docker_runtime.image_id,
							network: docker_runtime.network,
						},
					)
				}
			}),
		}),
	})
}
