use std::collections::HashMap;

use proto::backend;
use rivet_api::models;
use rivet_operation::prelude::*;

use crate::{ApiFrom, ApiInto, ApiTryFrom, ApiTryInto};
use serde_json::to_value;

impl ApiTryFrom<backend::dynamic_servers::Server> for models::ServersServer {
	type Error = GlobalError;
	fn api_try_from(
		value: backend::dynamic_servers::Server,
	) -> GlobalResult<models::ServersServer> {
		Ok(models::ServersServer {
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

impl ApiFrom<models::ServersResources> for backend::dynamic_servers::ServerResources {
	fn api_from(value: models::ServersResources) -> backend::dynamic_servers::ServerResources {
		backend::dynamic_servers::ServerResources {
			cpu_millicores: value.cpu,
			memory_mib: value.memory,
		}
	}
}

impl ApiFrom<backend::dynamic_servers::ServerResources> for models::ServersResources {
	fn api_from(value: backend::dynamic_servers::ServerResources) -> models::ServersResources {
		models::ServersResources {
			cpu: value.cpu_millicores,
			memory: value.memory_mib,
		}
	}
}

impl ApiTryFrom<models::ServersRuntime>
	for backend::pkg::dynamic_servers::server_create::request::Runtime
{
	type Error = GlobalError;

	fn api_try_from(
		value: models::ServersRuntime,
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

impl ApiTryFrom<backend::dynamic_servers::server::Runtime> for models::ServersRuntime {
	type Error = GlobalError;

	fn api_try_from(
		value: backend::dynamic_servers::server::Runtime,
	) -> GlobalResult<models::ServersRuntime> {
		match value {
			backend::dynamic_servers::server::Runtime::DockerRuntime(docker_runtime) => {
				Ok(models::ServersRuntime {
					docker: Some(Box::new(models::ServersDockerRuntime {
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

impl ApiTryFrom<models::ServersDockerNetwork> for backend::dynamic_servers::DockerNetwork {
	type Error = GlobalError;

	fn api_try_from(
		value: models::ServersDockerNetwork,
	) -> GlobalResult<backend::dynamic_servers::DockerNetwork> {
		Ok(backend::dynamic_servers::DockerNetwork {
			mode: backend::dynamic_servers::DockerNetworkMode::api_from(
				value.mode.unwrap_or_default(),
			) as i32,
			ports: unwrap!(value
				.ports
				.into_iter()
				.map(|(s, p)| Ok((s, p.api_try_into()?)))
				.collect::<GlobalResult<HashMap<_, _>>>()),
		})
	}
}

impl ApiTryInto<models::ServersDockerNetwork> for backend::dynamic_servers::DockerNetwork {
	type Error = GlobalError;

	fn api_try_into(self) -> GlobalResult<models::ServersDockerNetwork> {
		Ok(models::ServersDockerNetwork {
			mode: Some(
				unwrap!(backend::dynamic_servers::DockerNetworkMode::from_i32(
					self.mode
				))
				.api_into(),
			),
			ports: self
				.ports
				.into_iter()
				.map(|(s, p)| Ok((s, p.api_try_into()?)))
				.collect::<GlobalResult<HashMap<_, _>>>()?,
		})
	}
}

impl ApiFrom<models::ServersDockerNetworkMode> for backend::dynamic_servers::DockerNetworkMode {
	fn api_from(
		value: models::ServersDockerNetworkMode,
	) -> backend::dynamic_servers::DockerNetworkMode {
		match value {
			models::ServersDockerNetworkMode::Bridge => {
				backend::dynamic_servers::DockerNetworkMode::Bridge
			}
			models::ServersDockerNetworkMode::Host => {
				backend::dynamic_servers::DockerNetworkMode::Host
			}
		}
	}
}

impl ApiFrom<backend::dynamic_servers::DockerNetworkMode> for models::ServersDockerNetworkMode {
	fn api_from(
		value: backend::dynamic_servers::DockerNetworkMode,
	) -> models::ServersDockerNetworkMode {
		match value {
			backend::dynamic_servers::DockerNetworkMode::Bridge => {
				models::ServersDockerNetworkMode::Bridge
			}
			backend::dynamic_servers::DockerNetworkMode::Host => {
				models::ServersDockerNetworkMode::Host
			}
		}
	}
}

impl ApiTryFrom<models::ServersDockerPort> for backend::dynamic_servers::DockerPort {
	type Error = GlobalError;

	fn api_try_from(
		value: models::ServersDockerPort,
	) -> GlobalResult<backend::dynamic_servers::DockerPort> {
		Ok(backend::dynamic_servers::DockerPort {
			port: value.port,
			routing: Some((*value.routing).api_try_into()?),
		})
	}
}

impl ApiTryFrom<backend::dynamic_servers::DockerPort> for models::ServersDockerPort {
	type Error = GlobalError;

	fn api_try_from(
		value: backend::dynamic_servers::DockerPort,
	) -> GlobalResult<models::ServersDockerPort> {
		Ok(models::ServersDockerPort {
			port: value.port,
			routing: Box::new(unwrap!(value.routing).api_try_into()?),
		})
	}
}

impl ApiTryFrom<models::ServersDockerPortRouting>
	for backend::dynamic_servers::docker_port::Routing
{
	type Error = GlobalError;

	fn api_try_from(
		value: models::ServersDockerPortRouting,
	) -> GlobalResult<backend::dynamic_servers::docker_port::Routing> {
		match (value.game_guard, value.host) {
			(Some(game_guard), None) => Ok(
				backend::dynamic_servers::docker_port::Routing::GameGuard((*game_guard).api_into()),
			),
			(None, Some(host)) => Ok(backend::dynamic_servers::docker_port::Routing::Host(
				(*host).api_into(),
			)),
			(None, None) => bail_with!(SERVERS_NO_PORT_ROUTERS),
			_ => bail_with!(SERVERS_MULTIPLE_PORT_ROUTERS),
		}
	}
}

impl ApiTryFrom<backend::dynamic_servers::docker_port::Routing>
	for models::ServersDockerPortRouting
{
	type Error = GlobalError;

	fn api_try_from(
		value: backend::dynamic_servers::docker_port::Routing,
	) -> GlobalResult<models::ServersDockerPortRouting> {
		match value {
			backend::dynamic_servers::docker_port::Routing::GameGuard(game_guard) => {
				Ok(models::ServersDockerPortRouting {
					game_guard: Some(Box::new(game_guard.api_try_into()?)),
					host: None,
				})
			}
			backend::dynamic_servers::docker_port::Routing::Host(host) => {
				Ok(models::ServersDockerPortRouting {
					game_guard: None,
					host: Some(Box::new(host.api_try_into()?)),
				})
			}
		}
	}
}

impl ApiFrom<models::ServersDockerGameGuardRouting>
	for backend::dynamic_servers::DockerGameGuardRouting
{
	fn api_from(
		value: models::ServersDockerGameGuardRouting,
	) -> backend::dynamic_servers::DockerGameGuardRouting {
		backend::dynamic_servers::DockerGameGuardRouting {
			protocol: backend::dynamic_servers::GameGuardProtocol::api_from(
				value.protocol.unwrap_or_default().into(),
			) as i32,
		}
	}
}

impl ApiTryFrom<backend::dynamic_servers::DockerGameGuardRouting>
	for models::ServersDockerGameGuardRouting
{
	type Error = GlobalError;

	fn api_try_from(
		value: backend::dynamic_servers::DockerGameGuardRouting,
	) -> GlobalResult<models::ServersDockerGameGuardRouting> {
		Ok(models::ServersDockerGameGuardRouting {
			protocol: Some(
				unwrap!(backend::dynamic_servers::GameGuardProtocol::from_i32(
					value.protocol
				))
				.api_into(),
			),
		})
	}
}

impl ApiFrom<models::ServersDockerHostRouting> for backend::dynamic_servers::DockerHostRouting {
	fn api_from(
		value: models::ServersDockerHostRouting,
	) -> backend::dynamic_servers::DockerHostRouting {
		backend::dynamic_servers::DockerHostRouting {
			protocol: backend::dynamic_servers::HostProtocol::api_from(
				value.protocol.unwrap_or_default().into(),
			) as i32,
		}
	}
}

impl ApiTryFrom<backend::dynamic_servers::DockerHostRouting> for models::ServersDockerHostRouting {
	type Error = GlobalError;

	fn api_try_from(
		value: backend::dynamic_servers::DockerHostRouting,
	) -> GlobalResult<models::ServersDockerHostRouting> {
		Ok(models::ServersDockerHostRouting {
			protocol: Some(
				unwrap!(backend::dynamic_servers::HostProtocol::from_i32(
					value.protocol
				))
				.api_into(),
			),
		})
	}
}

impl ApiFrom<models::ServersGameGuardProtocol> for backend::dynamic_servers::GameGuardProtocol {
	fn api_from(
		value: models::ServersGameGuardProtocol,
	) -> backend::dynamic_servers::GameGuardProtocol {
		match value {
			models::ServersGameGuardProtocol::Udp => {
				backend::dynamic_servers::GameGuardProtocol::Udp
			}
			models::ServersGameGuardProtocol::Tcp => {
				backend::dynamic_servers::GameGuardProtocol::Tcp
			}
			models::ServersGameGuardProtocol::Http => {
				backend::dynamic_servers::GameGuardProtocol::Http
			}
			models::ServersGameGuardProtocol::Https => {
				backend::dynamic_servers::GameGuardProtocol::Https
			}
			models::ServersGameGuardProtocol::TcpTls => {
				backend::dynamic_servers::GameGuardProtocol::TcpTls
			}
		}
	}
}

impl ApiFrom<backend::dynamic_servers::GameGuardProtocol> for models::ServersGameGuardProtocol {
	fn api_from(
		value: backend::dynamic_servers::GameGuardProtocol,
	) -> models::ServersGameGuardProtocol {
		match value {
			backend::dynamic_servers::GameGuardProtocol::Udp => {
				models::ServersGameGuardProtocol::Udp
			}
			backend::dynamic_servers::GameGuardProtocol::Tcp => {
				models::ServersGameGuardProtocol::Tcp
			}
			backend::dynamic_servers::GameGuardProtocol::Http => {
				models::ServersGameGuardProtocol::Http
			}
			backend::dynamic_servers::GameGuardProtocol::Https => {
				models::ServersGameGuardProtocol::Https
			}
			backend::dynamic_servers::GameGuardProtocol::TcpTls => {
				models::ServersGameGuardProtocol::TcpTls
			}
		}
	}
}

impl ApiFrom<models::ServersHostProtocol> for backend::dynamic_servers::HostProtocol {
	fn api_from(value: models::ServersHostProtocol) -> backend::dynamic_servers::HostProtocol {
		match value {
			models::ServersHostProtocol::Udp => backend::dynamic_servers::HostProtocol::HostUdp,
			models::ServersHostProtocol::Tcp => backend::dynamic_servers::HostProtocol::HostTcp,
		}
	}
}

impl ApiFrom<backend::dynamic_servers::HostProtocol> for models::ServersHostProtocol {
	fn api_from(value: backend::dynamic_servers::HostProtocol) -> models::ServersHostProtocol {
		match value {
			backend::dynamic_servers::HostProtocol::HostUdp => models::ServersHostProtocol::Udp,
			backend::dynamic_servers::HostProtocol::HostTcp => models::ServersHostProtocol::Tcp,
		}
	}
}
