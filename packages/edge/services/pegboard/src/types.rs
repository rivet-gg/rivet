use std::{collections::HashMap, fmt};

use chirp_workflow::prelude::*;
use rivet_api::models;
use rivet_convert::{ApiFrom, ApiInto, ApiTryFrom};
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::FromRepr;

#[derive(Debug, Clone)]
pub struct Actor {
	pub actor_id: util::Id,
	pub env_id: Uuid,
	pub tags: HashMap<String, String>,
	pub image_id: Uuid,

	pub create_ts: i64,
	pub start_ts: Option<i64>,
	pub connectable_ts: Option<i64>,
	pub destroy_ts: Option<i64>,

	pub lifecycle: ActorLifecycle,
	pub resources: Option<ActorResources>,
	pub args: Vec<String>,
	pub network_mode: NetworkMode,
	pub environment: HashMap<String, String>,
	pub network_ports: HashMap<String, Port>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ActorResources {
	pub cpu_millicores: u32,
	pub memory_mib: u32,
}

impl ActorResources {
	pub fn default_isolate() -> Self {
		ActorResources {
			// cpu_millicores: 125,
			// memory_mib: 128,
			cpu_millicores: 250,
			memory_mib: 256,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ActorLifecycle {
	pub kill_timeout_ms: i64,
	pub durable: bool,
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum NetworkMode {
	Bridge = 0,
	Host = 1,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct Port {
	// Null when using host networking since one is automatically assigned
	pub internal_port: Option<i32>,
	pub public_hostname: Option<String>,
	pub public_port: Option<i32>,
	pub public_path: Option<String>,
	pub routing: Routing,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum Routing {
	GameGuard { protocol: GameGuardProtocol },
	Host { protocol: HostProtocol },
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum GameGuardProtocol {
	Http = 0,
	Https = 1,
	Tcp = 2,
	TcpTls = 3,
	Udp = 4,
}

impl fmt::Display for GameGuardProtocol {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			GameGuardProtocol::Http => write!(f, "http"),
			GameGuardProtocol::Https => write!(f, "https"),
			GameGuardProtocol::Tcp => write!(f, "tcp"),
			GameGuardProtocol::TcpTls => write!(f, "tcps"),
			GameGuardProtocol::Udp => write!(f, "udp"),
		}
	}
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum HostProtocol {
	Tcp = 0,
	Udp = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
	pub game_id: Uuid,
	pub host_networking_enabled: bool,
	pub root_user_enabled: bool,
}

impl GameConfig {
	pub fn default(game_id: Uuid) -> Self {
		GameConfig {
			game_id,
			host_networking_enabled: false,
			root_user_enabled: false,
		}
	}
}

/// Determines how port endpoints are returned.
#[derive(Debug, Copy, Clone, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum EndpointType {
	#[serde(rename = "hostname")]
	Hostname = 0,
	#[serde(rename = "path")]
	Path = 1,
}

impl EndpointType {
	pub fn default_for_guard_public_hostname(
		hostname: &cluster::types::GuardPublicHostname,
	) -> Self {
		match hostname {
			cluster::types::GuardPublicHostname::DnsParent(_) => Self::Hostname,
			cluster::types::GuardPublicHostname::Static(_) => Self::Path,
		}
	}
}

impl fmt::Display for EndpointType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			EndpointType::Hostname => write!(f, "hostname"),
			EndpointType::Path => write!(f, "path"),
		}
	}
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum LogsStreamType {
	StdOut = 0,
	StdErr = 1,
}

pub fn convert_actor_to_api(
	value: Actor,
	datacenter: &cluster::types::Datacenter,
) -> GlobalResult<models::ActorsActor> {
	Ok(models::ActorsActor {
		id: value.actor_id.to_string(),
		region: datacenter.name_id.clone(),
		created_at: util::timestamp::to_string(value.create_ts)?,
		// `started_at` -> `connectable_ts` is intentional. We don't expose the internal
		// workings of actors to the API, so we need to return the timestamp at which the server can
		// actually do anything useful.
		started_at: value
			.connectable_ts
			.map(util::timestamp::to_string)
			.transpose()?,
		destroyed_at: value
			.destroy_ts
			.map(util::timestamp::to_string)
			.transpose()?,
		tags: Some(serde_json::to_value(value.tags)?),
		runtime: Box::new(models::ActorsRuntime {
			build: value.image_id,
			arguments: Some(value.args),
			environment: Some(value.environment),
		}),
		network: Box::new(models::ActorsNetwork {
			mode: value.network_mode.api_into(),
			ports: value
				.network_ports
				.into_iter()
				.map(|(s, p)| (s, p.api_into()))
				.collect::<HashMap<_, _>>(),
		}),
		lifecycle: Box::new(value.lifecycle.api_into()),
	})
}

impl ApiFrom<models::ActorsLifecycle> for ActorLifecycle {
	fn api_from(value: models::ActorsLifecycle) -> ActorLifecycle {
		ActorLifecycle {
			kill_timeout_ms: value.kill_timeout.unwrap_or_default(),
			durable: value.durable.unwrap_or_default(),
		}
	}
}

impl ApiFrom<ActorLifecycle> for models::ActorsLifecycle {
	fn api_from(value: ActorLifecycle) -> models::ActorsLifecycle {
		models::ActorsLifecycle {
			kill_timeout: Some(value.kill_timeout_ms),
			durable: Some(value.durable),
		}
	}
}

impl ApiFrom<models::ActorsNetworkMode> for NetworkMode {
	fn api_from(value: models::ActorsNetworkMode) -> NetworkMode {
		match value {
			models::ActorsNetworkMode::Bridge => NetworkMode::Bridge,
			models::ActorsNetworkMode::Host => NetworkMode::Host,
		}
	}
}

impl ApiFrom<NetworkMode> for models::ActorsNetworkMode {
	fn api_from(value: NetworkMode) -> models::ActorsNetworkMode {
		match value {
			NetworkMode::Bridge => models::ActorsNetworkMode::Bridge,
			NetworkMode::Host => models::ActorsNetworkMode::Host,
		}
	}
}

impl ApiFrom<Port> for models::ActorsPort {
	fn api_from(value: Port) -> models::ActorsPort {
		let (protocol, routing, url) = match &value.routing {
			Routing::GameGuard { protocol } => {
				let url = match (
					protocol,
					value.public_hostname.as_ref(),
					value.public_port,
					value.public_path.as_ref(),
				) {
					(GameGuardProtocol::Http, Some(hostname), Some(80) | None, path) => Some(
						format!("{protocol}://{hostname}{}", util::format::OptDisplay(path)),
					),
					(GameGuardProtocol::Https, Some(hostname), Some(443) | None, path) => Some(
						format!("{protocol}://{hostname}{}", util::format::OptDisplay(path)),
					),
					(
						GameGuardProtocol::Http | GameGuardProtocol::Https,
						Some(hostname),
						Some(port),
						path,
					) => Some(format!(
						"{protocol}://{hostname}:{port}{}",
						util::format::OptDisplay(path)
					)),
					(_protocol, Some(hostname), Some(port), path) => Some(format!(
						"{hostname}:{port}{}",
						util::format::OptDisplay(path)
					)),
					(_protocol, Some(hostname), None, path) => {
						Some(format!("{hostname}{}", util::format::OptDisplay(path)))
					}
					_ => None,
				};

				(
					(*protocol).api_into(),
					models::ActorsPortRouting {
						guard: Some(json!({})),
						..Default::default()
					},
					url,
				)
			}
			Routing::Host { protocol } => (
				(*protocol).api_into(),
				models::ActorsPortRouting {
					host: Some(json!({})),
					..Default::default()
				},
				None,
			),
		};

		models::ActorsPort {
			protocol,
			internal_port: value.internal_port,
			hostname: value.public_hostname,
			port: value.public_port,
			path: value.public_path,
			url,
			routing: Box::new(routing),
		}
	}
}

impl ApiFrom<models::ActorsPortProtocol> for GameGuardProtocol {
	fn api_from(value: models::ActorsPortProtocol) -> GameGuardProtocol {
		match value {
			models::ActorsPortProtocol::Udp => GameGuardProtocol::Udp,
			models::ActorsPortProtocol::Tcp => GameGuardProtocol::Tcp,
			models::ActorsPortProtocol::Http => GameGuardProtocol::Http,
			models::ActorsPortProtocol::Https => GameGuardProtocol::Https,
			models::ActorsPortProtocol::TcpTls => GameGuardProtocol::TcpTls,
		}
	}
}

impl ApiFrom<GameGuardProtocol> for models::ActorsPortProtocol {
	fn api_from(value: GameGuardProtocol) -> models::ActorsPortProtocol {
		match value {
			GameGuardProtocol::Udp => models::ActorsPortProtocol::Udp,
			GameGuardProtocol::Tcp => models::ActorsPortProtocol::Tcp,
			GameGuardProtocol::Http => models::ActorsPortProtocol::Http,
			GameGuardProtocol::Https => models::ActorsPortProtocol::Https,
			GameGuardProtocol::TcpTls => models::ActorsPortProtocol::TcpTls,
		}
	}
}

impl ApiTryFrom<models::ActorsPortProtocol> for HostProtocol {
	type Error = GlobalError;
	fn api_try_from(value: models::ActorsPortProtocol) -> GlobalResult<HostProtocol> {
		Ok(match value {
			models::ActorsPortProtocol::Udp => HostProtocol::Udp,
			models::ActorsPortProtocol::Tcp => HostProtocol::Tcp,
			_ => {
				bail_with!(
					ACTOR_FAILED_TO_CREATE,
					error = "Host port protocol must be either TCP or UDP."
				);
			}
		})
	}
}

impl ApiFrom<HostProtocol> for models::ActorsPortProtocol {
	fn api_from(value: HostProtocol) -> models::ActorsPortProtocol {
		match value {
			HostProtocol::Udp => models::ActorsPortProtocol::Udp,
			HostProtocol::Tcp => models::ActorsPortProtocol::Tcp,
		}
	}
}

impl ApiFrom<models::ActorsEndpointType> for EndpointType {
	fn api_from(value: models::ActorsEndpointType) -> EndpointType {
		match value {
			models::ActorsEndpointType::Hostname => EndpointType::Hostname,
			models::ActorsEndpointType::Path => EndpointType::Path,
		}
	}
}

impl ApiFrom<EndpointType> for models::ActorsEndpointType {
	fn api_from(value: EndpointType) -> models::ActorsEndpointType {
		match value {
			EndpointType::Hostname => models::ActorsEndpointType::Hostname,
			EndpointType::Path => models::ActorsEndpointType::Path,
		}
	}
}

// MARK: Containers
pub fn convert_container_to_api(
	value: Actor,
	datacenter: &cluster::types::Datacenter,
) -> GlobalResult<models::ContainersContainer> {
	Ok(models::ContainersContainer {
		id: value.actor_id.to_string(),
		region: datacenter.name_id.clone(),
		created_at: util::timestamp::to_string(value.create_ts)?,
		// `started_at` -> `connectable_ts` is intentional. We don't expose the internal
		// workings of actors to the API, so we need to return the timestamp at which the server can
		// actually do anything useful.
		started_at: value
			.connectable_ts
			.map(util::timestamp::to_string)
			.transpose()?,
		destroyed_at: value
			.destroy_ts
			.map(util::timestamp::to_string)
			.transpose()?,
		tags: Some(serde_json::to_value(value.tags)?),
		runtime: Box::new(models::ContainersRuntime {
			build: value.image_id,
			arguments: Some(value.args),
			environment: Some(value.environment),
		}),
		network: Box::new(models::ContainersNetwork {
			mode: value.network_mode.api_into(),
			ports: value
				.network_ports
				.into_iter()
				.map(|(s, p)| (s, p.api_into()))
				.collect::<HashMap<_, _>>(),
		}),
		lifecycle: Box::new(value.lifecycle.api_into()),
		resources: Box::new(unwrap!(value.resources, "container should have resources").api_into()),
	})
}

impl ApiFrom<models::ContainersResources> for ActorResources {
	fn api_from(value: models::ContainersResources) -> ActorResources {
		ActorResources {
			cpu_millicores: value.cpu as u32,
			memory_mib: value.memory as u32,
		}
	}
}

impl ApiFrom<ActorResources> for models::ContainersResources {
	fn api_from(value: ActorResources) -> models::ContainersResources {
		models::ContainersResources {
			cpu: value.cpu_millicores as i32,
			memory: value.memory_mib as i32,
		}
	}
}

impl ApiFrom<models::ContainersLifecycle> for ActorLifecycle {
	fn api_from(value: models::ContainersLifecycle) -> ActorLifecycle {
		ActorLifecycle {
			kill_timeout_ms: value.kill_timeout.unwrap_or_default(),
			durable: value.durable.unwrap_or_default(),
		}
	}
}

impl ApiFrom<ActorLifecycle> for models::ContainersLifecycle {
	fn api_from(value: ActorLifecycle) -> models::ContainersLifecycle {
		models::ContainersLifecycle {
			kill_timeout: Some(value.kill_timeout_ms),
			durable: Some(value.durable),
		}
	}
}

impl ApiFrom<models::ContainersNetworkMode> for NetworkMode {
	fn api_from(value: models::ContainersNetworkMode) -> NetworkMode {
		match value {
			models::ContainersNetworkMode::Bridge => NetworkMode::Bridge,
			models::ContainersNetworkMode::Host => NetworkMode::Host,
		}
	}
}

impl ApiFrom<NetworkMode> for models::ContainersNetworkMode {
	fn api_from(value: NetworkMode) -> models::ContainersNetworkMode {
		match value {
			NetworkMode::Bridge => models::ContainersNetworkMode::Bridge,
			NetworkMode::Host => models::ContainersNetworkMode::Host,
		}
	}
}

impl ApiFrom<Port> for models::ContainersPort {
	fn api_from(value: Port) -> models::ContainersPort {
		let (protocol, routing, url) = match &value.routing {
			Routing::GameGuard { protocol } => {
				let url = match (
					protocol,
					value.public_hostname.as_ref(),
					value.public_port,
					value.public_path.as_ref(),
				) {
					(GameGuardProtocol::Http, Some(hostname), Some(80) | None, path) => Some(
						format!("{protocol}://{hostname}{}", util::format::OptDisplay(path)),
					),
					(GameGuardProtocol::Https, Some(hostname), Some(443) | None, path) => Some(
						format!("{protocol}://{hostname}{}", util::format::OptDisplay(path)),
					),
					(
						GameGuardProtocol::Http | GameGuardProtocol::Https,
						Some(hostname),
						Some(port),
						path,
					) => Some(format!(
						"{protocol}://{hostname}:{port}{}",
						util::format::OptDisplay(path)
					)),
					(_protocol, Some(hostname), Some(port), path) => Some(format!(
						"{hostname}:{port}{}",
						util::format::OptDisplay(path)
					)),
					(_protocol, Some(hostname), None, path) => {
						Some(format!("{hostname}{}", util::format::OptDisplay(path)))
					}
					_ => None,
				};

				(
					(*protocol).api_into(),
					models::ContainersPortRouting {
						guard: Some(json!({})),
						..Default::default()
					},
					url,
				)
			}
			Routing::Host { protocol } => (
				(*protocol).api_into(),
				models::ContainersPortRouting {
					host: Some(json!({})),
					..Default::default()
				},
				None,
			),
		};

		models::ContainersPort {
			protocol,
			internal_port: value.internal_port,
			hostname: value.public_hostname,
			port: value.public_port,
			path: value.public_path,
			url,
			routing: Box::new(routing),
		}
	}
}

impl ApiFrom<models::ContainersPortProtocol> for GameGuardProtocol {
	fn api_from(value: models::ContainersPortProtocol) -> GameGuardProtocol {
		match value {
			models::ContainersPortProtocol::Udp => GameGuardProtocol::Udp,
			models::ContainersPortProtocol::Tcp => GameGuardProtocol::Tcp,
			models::ContainersPortProtocol::Http => GameGuardProtocol::Http,
			models::ContainersPortProtocol::Https => GameGuardProtocol::Https,
			models::ContainersPortProtocol::TcpTls => GameGuardProtocol::TcpTls,
		}
	}
}

impl ApiFrom<GameGuardProtocol> for models::ContainersPortProtocol {
	fn api_from(value: GameGuardProtocol) -> models::ContainersPortProtocol {
		match value {
			GameGuardProtocol::Udp => models::ContainersPortProtocol::Udp,
			GameGuardProtocol::Tcp => models::ContainersPortProtocol::Tcp,
			GameGuardProtocol::Http => models::ContainersPortProtocol::Http,
			GameGuardProtocol::Https => models::ContainersPortProtocol::Https,
			GameGuardProtocol::TcpTls => models::ContainersPortProtocol::TcpTls,
		}
	}
}

impl ApiTryFrom<models::ContainersPortProtocol> for HostProtocol {
	type Error = GlobalError;
	fn api_try_from(value: models::ContainersPortProtocol) -> GlobalResult<HostProtocol> {
		Ok(match value {
			models::ContainersPortProtocol::Udp => HostProtocol::Udp,
			models::ContainersPortProtocol::Tcp => HostProtocol::Tcp,
			_ => {
				bail_with!(
					CONTAINER_FAILED_TO_CREATE,
					error = "Host port protocol must be either TCP or UDP."
				);
			}
		})
	}
}

impl ApiFrom<HostProtocol> for models::ContainersPortProtocol {
	fn api_from(value: HostProtocol) -> models::ContainersPortProtocol {
		match value {
			HostProtocol::Udp => models::ContainersPortProtocol::Udp,
			HostProtocol::Tcp => models::ContainersPortProtocol::Tcp,
		}
	}
}

impl ApiFrom<models::ContainersEndpointType> for EndpointType {
	fn api_from(value: models::ContainersEndpointType) -> EndpointType {
		match value {
			models::ContainersEndpointType::Hostname => EndpointType::Hostname,
			models::ContainersEndpointType::Path => EndpointType::Path,
		}
	}
}

impl ApiFrom<EndpointType> for models::ContainersEndpointType {
	fn api_from(value: EndpointType) -> models::ContainersEndpointType {
		match value {
			EndpointType::Hostname => models::ContainersEndpointType::Hostname,
			EndpointType::Path => models::ContainersEndpointType::Path,
		}
	}
}

// MARK: V1
pub mod v1 {
	use super::*;

	pub fn convert_actor_to_api(
		value: Actor,
		datacenter: &cluster::types::Datacenter,
	) -> GlobalResult<models::ActorsV1Actor> {
		Ok(models::ActorsV1Actor {
			id: unwrap!(value.actor_id.as_v0(), "cannot convert new actor to v1"),
			region: datacenter.name_id.clone(),
			created_at: util::timestamp::to_string(value.create_ts)?,
			// `started_at` -> `connectable_ts` is intentional. We don't expose the internal
			// workings of actors to the API, so we need to return the timestamp at which the server can
			// actually do anything useful.
			started_at: value
				.connectable_ts
				.map(util::timestamp::to_string)
				.transpose()?,
			destroyed_at: value
				.destroy_ts
				.map(util::timestamp::to_string)
				.transpose()?,
			tags: Some(serde_json::to_value(value.tags)?),
			runtime: Box::new(models::ActorsV1Runtime {
				build: value.image_id,
				arguments: Some(value.args),
				environment: Some(value.environment),
			}),
			network: Box::new(models::ActorsV1Network {
				mode: value.network_mode.api_into(),
				ports: value
					.network_ports
					.into_iter()
					.map(|(s, p)| (s, p.api_into()))
					.collect::<HashMap<_, _>>(),
			}),
			lifecycle: Box::new(value.lifecycle.api_into()),
			resources: value.resources.map(ApiInto::api_into).map(Box::new),
		})
	}

	impl ApiFrom<models::ActorsV1Resources> for ActorResources {
		fn api_from(value: models::ActorsV1Resources) -> ActorResources {
			ActorResources {
				cpu_millicores: value.cpu as u32,
				memory_mib: value.memory as u32,
			}
		}
	}

	impl ApiFrom<ActorResources> for models::ActorsV1Resources {
		fn api_from(value: ActorResources) -> models::ActorsV1Resources {
			models::ActorsV1Resources {
				cpu: value.cpu_millicores as i32,
				memory: value.memory_mib as i32,
			}
		}
	}

	impl ApiFrom<models::ActorsV1Lifecycle> for ActorLifecycle {
		fn api_from(value: models::ActorsV1Lifecycle) -> ActorLifecycle {
			ActorLifecycle {
				kill_timeout_ms: value.kill_timeout.unwrap_or_default(),
				durable: value.durable.unwrap_or_default(),
			}
		}
	}

	impl ApiFrom<ActorLifecycle> for models::ActorsV1Lifecycle {
		fn api_from(value: ActorLifecycle) -> models::ActorsV1Lifecycle {
			models::ActorsV1Lifecycle {
				kill_timeout: Some(value.kill_timeout_ms),
				durable: Some(value.durable),
			}
		}
	}

	impl ApiFrom<models::ActorsV1NetworkMode> for NetworkMode {
		fn api_from(value: models::ActorsV1NetworkMode) -> NetworkMode {
			match value {
				models::ActorsV1NetworkMode::Bridge => NetworkMode::Bridge,
				models::ActorsV1NetworkMode::Host => NetworkMode::Host,
			}
		}
	}

	impl ApiFrom<NetworkMode> for models::ActorsV1NetworkMode {
		fn api_from(value: NetworkMode) -> models::ActorsV1NetworkMode {
			match value {
				NetworkMode::Bridge => models::ActorsV1NetworkMode::Bridge,
				NetworkMode::Host => models::ActorsV1NetworkMode::Host,
			}
		}
	}

	impl ApiFrom<Port> for models::ActorsV1Port {
		fn api_from(value: Port) -> models::ActorsV1Port {
			let (protocol, routing, url) = match &value.routing {
				Routing::GameGuard { protocol } => {
					let url = match (
						protocol,
						value.public_hostname.as_ref(),
						value.public_port,
						value.public_path.as_ref(),
					) {
						(GameGuardProtocol::Http, Some(hostname), Some(80) | None, path) => Some(
							format!("{protocol}://{hostname}{}", util::format::OptDisplay(path)),
						),
						(GameGuardProtocol::Https, Some(hostname), Some(443) | None, path) => Some(
							format!("{protocol}://{hostname}{}", util::format::OptDisplay(path)),
						),
						(
							GameGuardProtocol::Http | GameGuardProtocol::Https,
							Some(hostname),
							Some(port),
							path,
						) => Some(format!(
							"{protocol}://{hostname}:{port}{}",
							util::format::OptDisplay(path)
						)),
						(_protocol, Some(hostname), Some(port), path) => Some(format!(
							"{hostname}:{port}{}",
							util::format::OptDisplay(path)
						)),
						(_protocol, Some(hostname), None, path) => {
							Some(format!("{hostname}{}", util::format::OptDisplay(path)))
						}
						_ => None,
					};

					(
						(*protocol).api_into(),
						models::ActorsV1PortRouting {
							guard: Some(json!({})),
							..Default::default()
						},
						url,
					)
				}
				Routing::Host { protocol } => (
					(*protocol).api_into(),
					models::ActorsV1PortRouting {
						host: Some(json!({})),
						..Default::default()
					},
					None,
				),
			};

			models::ActorsV1Port {
				protocol,
				internal_port: value.internal_port,
				hostname: value.public_hostname,
				port: value.public_port,
				path: value.public_path,
				url,
				routing: Box::new(routing),
			}
		}
	}

	impl ApiFrom<models::ActorsV1PortProtocol> for GameGuardProtocol {
		fn api_from(value: models::ActorsV1PortProtocol) -> GameGuardProtocol {
			match value {
				models::ActorsV1PortProtocol::Udp => GameGuardProtocol::Udp,
				models::ActorsV1PortProtocol::Tcp => GameGuardProtocol::Tcp,
				models::ActorsV1PortProtocol::Http => GameGuardProtocol::Http,
				models::ActorsV1PortProtocol::Https => GameGuardProtocol::Https,
				models::ActorsV1PortProtocol::TcpTls => GameGuardProtocol::TcpTls,
			}
		}
	}

	impl ApiFrom<GameGuardProtocol> for models::ActorsV1PortProtocol {
		fn api_from(value: GameGuardProtocol) -> models::ActorsV1PortProtocol {
			match value {
				GameGuardProtocol::Udp => models::ActorsV1PortProtocol::Udp,
				GameGuardProtocol::Tcp => models::ActorsV1PortProtocol::Tcp,
				GameGuardProtocol::Http => models::ActorsV1PortProtocol::Http,
				GameGuardProtocol::Https => models::ActorsV1PortProtocol::Https,
				GameGuardProtocol::TcpTls => models::ActorsV1PortProtocol::TcpTls,
			}
		}
	}

	impl ApiTryFrom<models::ActorsV1PortProtocol> for HostProtocol {
		type Error = GlobalError;
		fn api_try_from(value: models::ActorsV1PortProtocol) -> GlobalResult<HostProtocol> {
			Ok(match value {
				models::ActorsV1PortProtocol::Udp => HostProtocol::Udp,
				models::ActorsV1PortProtocol::Tcp => HostProtocol::Tcp,
				_ => {
					bail_with!(
						ACTOR_FAILED_TO_CREATE,
						error = "Host port protocol must be either TCP or UDP."
					);
				}
			})
		}
	}

	impl ApiFrom<HostProtocol> for models::ActorsV1PortProtocol {
		fn api_from(value: HostProtocol) -> models::ActorsV1PortProtocol {
			match value {
				HostProtocol::Udp => models::ActorsV1PortProtocol::Udp,
				HostProtocol::Tcp => models::ActorsV1PortProtocol::Tcp,
			}
		}
	}

	impl ApiFrom<models::ActorsV1EndpointType> for EndpointType {
		fn api_from(value: models::ActorsV1EndpointType) -> EndpointType {
			match value {
				models::ActorsV1EndpointType::Hostname => EndpointType::Hostname,
				models::ActorsV1EndpointType::Path => EndpointType::Path,
			}
		}
	}

	impl ApiFrom<EndpointType> for models::ActorsV1EndpointType {
		fn api_from(value: EndpointType) -> models::ActorsV1EndpointType {
			match value {
				EndpointType::Hostname => models::ActorsV1EndpointType::Hostname,
				EndpointType::Path => models::ActorsV1EndpointType::Path,
			}
		}
	}
}
