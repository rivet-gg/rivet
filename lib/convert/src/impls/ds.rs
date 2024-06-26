use std::collections::HashMap;

use proto::backend;
use rivet_api::models;
use rivet_operation::prelude::*;

use crate::{ApiFrom, ApiInto, ApiTryFrom, ApiTryInto};
use serde_json::to_value;

impl ApiTryFrom<backend::dynamic_servers::Server> for models::DynamicServersServer {
	type Error = GlobalError;
	fn api_try_from(value: backend::dynamic_servers::Server) -> GlobalResult<models::DynamicServersServer> {
		Ok(models::DynamicServersServer {
			cluster_id: unwrap!(value.cluster_id).as_uuid(),
			create_ts: value.create_ts,
			datacenter_id: unwrap!(value.datacenter_id).as_uuid(),
			destroy_ts: value.destroy_ts,
			game_id: unwrap!(value.game_id).as_uuid(),
			kill_timeout: Some(value.kill_timeout_ms),
			metadata: Some(to_value(value.metadata).unwrap()),
			resources: Box::new(unwrap!(value.resources).api_into()),
			runtime: Box::new(unwrap!(value.runtime).api_try_into()?),
			server_id: unwrap!(value.server_id).as_uuid(),
		})
	}
}

impl ApiFrom<models::DynamicServersResources> for backend::dynamic_servers::ServerResources {
	fn api_from(value: models::DynamicServersResources) -> backend::dynamic_servers::ServerResources {
		backend::dynamic_servers::ServerResources {
			cpu_millicores: value.cpu,
			memory_mib: value.memory,
		}
	}
}

impl ApiFrom<backend::dynamic_servers::ServerResources> for models::DynamicServersResources {
	fn api_from(value: backend::dynamic_servers::ServerResources) -> models::DynamicServersResources {
		models::DynamicServersResources {
			cpu: value.cpu_millicores,
			memory: value.memory_mib,
		}
	}
}

impl ApiTryFrom<models::DynamicServersRuntime> for backend::pkg::dynamic_servers::server_create::request::Runtime {
	type Error = GlobalError;

	fn api_try_from(
		value: models::DynamicServersRuntime,
	) -> GlobalResult<backend::pkg::dynamic_servers::server_create::request::Runtime> {
		match (value.docker,) {
			(Some(docker_runtime),) => Ok(
				backend::pkg::dynamic_servers::server_create::request::Runtime::DockerRuntime(
					backend::dynamic_servers::DockerRuntime {
						args: docker_runtime.args.unwrap_or_default(),
						environment: docker_runtime.environment.unwrap_or_default(),
						image_id: Some(docker_runtime.image_id.into()),
						network: Some(backend::dynamic_servers::DockerNetwork::api_try_from(
							*docker_runtime.network,
						)?),
					},
				),
			),
			(None,) => bail_with!(SERVERS_NO_RUNTIME),
			// This needs to be added if other runtimes are added
			// _ => bail_with!(SERVERS_MULTIPLE_RUNTIMES),
		}
	}
}

impl ApiTryFrom<backend::dynamic_servers::server::Runtime> for models::DynamicServersRuntime {
	type Error = GlobalError;

	fn api_try_from(
		value: backend::dynamic_servers::server::Runtime,
	) -> GlobalResult<models::DynamicServersRuntime> {
		match value {
			backend::dynamic_servers::server::Runtime::DockerRuntime(docker_runtime) => {
				Ok(models::DynamicServersRuntime {
					docker: Some(Box::new(models::DynamicServersDockerRuntime {
						args: Some(docker_runtime.args),
						environment: Some(docker_runtime.environment),
						image_id: unwrap!(docker_runtime.image_id).as_uuid(),
						network: Box::new(unwrap!(docker_runtime.network).api_try_into()?),
					})),
				})
			}
		}
	}
}

impl ApiTryFrom<models::DynamicServersDockerNetwork> for backend::dynamic_servers::DockerNetwork {
	type Error = GlobalError;

	fn api_try_from(
		value: models::DynamicServersDockerNetwork,
	) -> GlobalResult<backend::dynamic_servers::DockerNetwork> {
		Ok(backend::dynamic_servers::DockerNetwork {
			mode: backend::dynamic_servers::DockerNetworkMode::api_from(value.mode.unwrap_or_default())
				as i32,
			ports: unwrap!(value
				.ports
				.into_iter()
				.map(|(s, p)| Ok((s, p.api_try_into()?)))
				.collect::<GlobalResult<HashMap<_, _>>>()),
		})
	}
}

impl ApiTryInto<models::DynamicServersDockerNetwork> for backend::dynamic_servers::DockerNetwork {
	type Error = GlobalError;

	fn api_try_into(self) -> GlobalResult<models::DynamicServersDockerNetwork> {
		Ok(models::DynamicServersDockerNetwork {
			mode: Some(
				unwrap!(backend::dynamic_servers::DockerNetworkMode::from_i32(self.mode)).api_into(),
			),
			ports: self
				.ports
				.into_iter()
				.map(|(s, p)| Ok((s, p.api_try_into()?)))
				.collect::<GlobalResult<HashMap<_, _>>>()?,
		})
	}
}

impl ApiFrom<models::DynamicServersDockerNetworkMode> for backend::dynamic_servers::DockerNetworkMode {
	fn api_from(value: models::DynamicServersDockerNetworkMode) -> backend::dynamic_servers::DockerNetworkMode {
		match value {
			models::DynamicServersDockerNetworkMode::Bridge => backend::dynamic_servers::DockerNetworkMode::Bridge,
			models::DynamicServersDockerNetworkMode::Host => backend::dynamic_servers::DockerNetworkMode::Host,
		}
	}
}

impl ApiFrom<backend::dynamic_servers::DockerNetworkMode> for models::DynamicServersDockerNetworkMode {
	fn api_from(value: backend::dynamic_servers::DockerNetworkMode) -> models::DynamicServersDockerNetworkMode {
		match value {
			backend::dynamic_servers::DockerNetworkMode::Bridge => models::DynamicServersDockerNetworkMode::Bridge,
			backend::dynamic_servers::DockerNetworkMode::Host => models::DynamicServersDockerNetworkMode::Host,
		}
	}
}

impl ApiTryFrom<models::DynamicServersDockerPort> for backend::dynamic_servers::DockerPort {
	type Error = GlobalError;

	fn api_try_from(
		value: models::DynamicServersDockerPort,
	) -> GlobalResult<backend::dynamic_servers::DockerPort> {
		Ok(backend::dynamic_servers::DockerPort {
			port: value.port,
			routing: Some((*value.routing).api_try_into()?),
		})
	}
}

impl ApiTryFrom<backend::dynamic_servers::DockerPort> for models::DynamicServersDockerPort {
	type Error = GlobalError;

	fn api_try_from(
		value: backend::dynamic_servers::DockerPort,
	) -> GlobalResult<models::DynamicServersDockerPort> {
		Ok(models::DynamicServersDockerPort {
			port: value.port,
			routing: Box::new(unwrap!(value.routing).api_try_into()?),
		})
	}
}

impl ApiTryFrom<models::DynamicServersDockerPortRouting> for backend::dynamic_servers::docker_port::Routing {
	type Error = GlobalError;

	fn api_try_from(
		value: models::DynamicServersDockerPortRouting,
	) -> GlobalResult<backend::dynamic_servers::docker_port::Routing> {
		match (value.game_guard, value.host) {
			(Some(game_guard), None) => Ok(backend::dynamic_servers::docker_port::Routing::GameGuard(
				(*game_guard).api_into(),
			)),
			(None, Some(_)) => Ok(backend::dynamic_servers::docker_port::Routing::Host(
				backend::dynamic_servers::DockerHostRouting {},
			)),
			(None, None) => bail_with!(SERVERS_NO_PORT_ROUTERS),
			_ => bail_with!(SERVERS_MULTIPLE_PORT_ROUTERS),
		}
	}
}

impl ApiTryFrom<backend::dynamic_servers::docker_port::Routing> for models::DynamicServersDockerPortRouting {
	type Error = GlobalError;

	fn api_try_from(
		value: backend::dynamic_servers::docker_port::Routing,
	) -> GlobalResult<models::DynamicServersDockerPortRouting> {
		match value {
			backend::dynamic_servers::docker_port::Routing::GameGuard(game_guard) => {
				Ok(models::DynamicServersDockerPortRouting {
					game_guard: Some(Box::new(game_guard.api_try_into()?)),
					host: None,
				})
			}
			backend::dynamic_servers::docker_port::Routing::Host(_) => {
				Ok(models::DynamicServersDockerPortRouting {
					game_guard: None,
					host: Some(to_value({})?),
				})
			}
		}
	}
}

impl ApiFrom<models::DynamicServersDockerGameGuardRouting> for backend::dynamic_servers::DockerGameGuardRouting {
	fn api_from(
		value: models::DynamicServersDockerGameGuardRouting,
	) -> backend::dynamic_servers::DockerGameGuardRouting {
		backend::dynamic_servers::DockerGameGuardRouting {
			protocol: backend::dynamic_servers::GameGuardProtocol::api_from(
				value.protocol.unwrap_or_default().into(),
			) as i32,
		}
	}
}

impl ApiTryFrom<backend::dynamic_servers::DockerGameGuardRouting>
	for models::DynamicServersDockerGameGuardRouting
{
	type Error = GlobalError;

	fn api_try_from(
		value: backend::dynamic_servers::DockerGameGuardRouting,
	) -> GlobalResult<models::DynamicServersDockerGameGuardRouting> {
		Ok(models::DynamicServersDockerGameGuardRouting {
			protocol: Some(
				unwrap!(backend::dynamic_servers::GameGuardProtocol::from_i32(
					value.protocol
				))
				.api_into(),
			),
		})
	}
}

impl ApiFrom<models::DynamicServersGameGuardProtocol> for backend::dynamic_servers::GameGuardProtocol {
	fn api_from(value: models::DynamicServersGameGuardProtocol) -> backend::dynamic_servers::GameGuardProtocol {
		match value {
			models::DynamicServersGameGuardProtocol::Udp => backend::dynamic_servers::GameGuardProtocol::Udp,
			models::DynamicServersGameGuardProtocol::Tcp => backend::dynamic_servers::GameGuardProtocol::Tcp,
			models::DynamicServersGameGuardProtocol::Http => backend::dynamic_servers::GameGuardProtocol::Http,
			models::DynamicServersGameGuardProtocol::Https => backend::dynamic_servers::GameGuardProtocol::Https,
			models::DynamicServersGameGuardProtocol::TcpTls => backend::dynamic_servers::GameGuardProtocol::TcpTls,
		}
	}
}

impl ApiFrom<backend::dynamic_servers::GameGuardProtocol> for models::DynamicServersGameGuardProtocol {
	fn api_from(value: backend::dynamic_servers::GameGuardProtocol) -> models::DynamicServersGameGuardProtocol {
		match value {
			backend::dynamic_servers::GameGuardProtocol::Udp => models::DynamicServersGameGuardProtocol::Udp,
			backend::dynamic_servers::GameGuardProtocol::Tcp => models::DynamicServersGameGuardProtocol::Tcp,
			backend::dynamic_servers::GameGuardProtocol::Http => models::DynamicServersGameGuardProtocol::Http,
			backend::dynamic_servers::GameGuardProtocol::Https => models::DynamicServersGameGuardProtocol::Https,
			backend::dynamic_servers::GameGuardProtocol::TcpTls => models::DynamicServersGameGuardProtocol::TcpTls,
		}
	}
}
