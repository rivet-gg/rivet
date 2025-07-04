use std::{collections::HashMap, fmt};

use chirp_workflow::prelude::*;
use rivet_api::models;
use rivet_convert::{ApiFrom, ApiInto, ApiTryFrom, ApiTryInto};
use serde_json::json;
use strum::FromRepr;

// NOTE: Do not change the serde case of this or else it will break workflow hashes
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, FromRepr)]
pub enum BuildKind {
	DockerImage = 0,
	OciBundle = 1,
	JavaScript = 2,
}

// NOTE: Do not change the serde case of this or else it will break workflow hashes
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, FromRepr)]
pub enum BuildCompression {
	None = 0,
	Lz4 = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
	pub build_id: Uuid,
	pub game_id: Option<Uuid>,
	pub env_id: Option<Uuid>,
	pub upload_id: Uuid,
	pub display_name: String,
	pub image_tag: String,
	pub create_ts: i64,
	pub kind: BuildKind,
	pub compression: BuildCompression,
	pub runtime: Option<BuildRuntime>,
	pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum BuildRuntime {
	Container {
		args: Vec<String>,
		environment: util::serde::HashableMap<String, String>,
		network_mode: BuildNetworkMode,
		ports: util::serde::HashableMap<String, BuildPort>,
	},
	Actor {
		args: Vec<String>,
		environment: util::serde::HashableMap<String, String>,
		network_mode: BuildNetworkMode,
		ports: util::serde::HashableMap<String, BuildPort>,
		resources: BuildResources,
		slots: u32,
	},
}

impl BuildRuntime {
	pub fn slots(&self) -> u32 {
		match self {
			BuildRuntime::Container { .. } => 1,
			BuildRuntime::Actor { slots, .. } => *slots,
		}
	}

	pub fn args(&self) -> &[String] {
		match self {
			BuildRuntime::Container { args, .. } => args,
			BuildRuntime::Actor { args, .. } => args,
		}
	}

	pub fn environment(&self) -> &util::serde::HashableMap<String, String> {
		match self {
			BuildRuntime::Container { environment, .. } => environment,
			BuildRuntime::Actor { environment, .. } => environment,
		}
	}

	pub fn network_mode(&self) -> BuildNetworkMode {
		match self {
			BuildRuntime::Container { network_mode, .. } => *network_mode,
			BuildRuntime::Actor { network_mode, .. } => *network_mode,
		}
	}

	pub fn ports(&self) -> &util::serde::HashableMap<String, BuildPort> {
		match self {
			BuildRuntime::Container { ports, .. } => ports,
			BuildRuntime::Actor { ports, .. } => ports,
		}
	}

	pub fn kind(&self) -> BuildRuntimeKind {
		match self {
			BuildRuntime::Container { .. } => BuildRuntimeKind::Container,
			BuildRuntime::Actor { .. } => BuildRuntimeKind::Actor,
		}
	}
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, FromRepr)]
#[serde(rename_all = "snake_case")]
pub enum BuildRuntimeKind {
	Container = 0,
	Actor = 1,
}

// TODO: These types are copied from pegboard types
#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum BuildNetworkMode {
	Bridge = 0,
	Host = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct BuildPort {
	// Null when using host networking since one is automatically assigned
	pub internal_port: Option<u16>,
	pub routing: PortRouting,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum PortRouting {
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

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct BuildResources {
	pub cpu_millicores: u32,
	pub memory_mib: u32,
}

impl ApiFrom<models::BuildsKind> for BuildKind {
	fn api_from(value: models::BuildsKind) -> BuildKind {
		match value {
			models::BuildsKind::DockerImage => BuildKind::DockerImage,
			models::BuildsKind::OciBundle => BuildKind::OciBundle,
			models::BuildsKind::Javascript => BuildKind::JavaScript,
		}
	}
}

impl ApiFrom<models::BuildsCompression> for BuildCompression {
	fn api_from(value: models::BuildsCompression) -> BuildCompression {
		match value {
			models::BuildsCompression::None => BuildCompression::None,
			models::BuildsCompression::Lz4 => BuildCompression::Lz4,
		}
	}
}
impl ApiTryFrom<BuildRuntime> for models::BuildsRuntime {
	type Error = GlobalError;

	fn api_try_from(value: BuildRuntime) -> GlobalResult<models::BuildsRuntime> {
		match value {
			BuildRuntime::Container {
				environment,
				network_mode,
				ports,
				..
			} => Ok(models::BuildsRuntime {
				container: Some(Box::new(models::BuildsRuntimeContainer {
					environment: Some(environment.into()),
					network_mode: Some(network_mode.api_into()),
					ports: Some(
						ports
							.into_iter()
							.map(|(s, p)| Ok((s, p.api_try_into()?)))
							.collect::<GlobalResult<HashMap<_, _>>>()?,
					),
				})),
				actor: None,
			}),
			BuildRuntime::Actor {
				environment,
				network_mode,
				ports,
				resources,
				slots,
				..
			} => Ok(models::BuildsRuntime {
				container: None,
				actor: Some(Box::new(models::BuildsRuntimeActor {
					environment: Some(environment.into()),
					network_mode: Some(network_mode.api_into()),
					ports: Some(
						ports
							.into_iter()
							.map(|(s, p)| Ok((s, p.api_try_into()?)))
							.collect::<GlobalResult<HashMap<_, _>>>()?,
					),
					resources: Box::new(resources.api_try_into()?),
					slots: slots.try_into()?,
				})),
			}),
		}
	}
}

impl ApiTryFrom<models::BuildsRuntime> for BuildRuntime {
	type Error = GlobalError;

	fn api_try_from(value: models::BuildsRuntime) -> GlobalResult<BuildRuntime> {
		match (value.container, value.actor) {
			(Some(container_runtime), None) => Ok(BuildRuntime::Container {
				args: Default::default(),
				environment: container_runtime
					.environment
					.clone()
					.unwrap_or_default()
					.into(),
				network_mode: container_runtime
					.network_mode
					.unwrap_or_default()
					.api_into(),
				ports: container_runtime
					.ports
					.clone()
					.unwrap_or_default()
					.into_iter()
					.map(|(s, port)| {
						let port = convert_port_request_from_api(&s, port)?;
						Ok((s, port))
					})
					.collect::<GlobalResult<util::serde::HashableMap<_, _>>>()?,
			}),
			(None, Some(actor_runtime)) => Ok(BuildRuntime::Actor {
				args: Default::default(),
				environment: actor_runtime.environment.clone().unwrap_or_default().into(),
				network_mode: actor_runtime.network_mode.unwrap_or_default().api_into(),
				ports: actor_runtime
					.ports
					.clone()
					.unwrap_or_default()
					.into_iter()
					.map(|(s, port)| {
						let port = convert_port_request_from_api(&s, port)?;
						Ok((s, port))
					})
					.collect::<GlobalResult<util::serde::HashableMap<_, _>>>()?,
				resources: (*actor_runtime.resources).api_try_into()?,
				slots: actor_runtime.slots.try_into()?,
			}),
			(Some(_), Some(_)) => {
				bail_with!(
					API_BAD_BODY,
					reason = "cannot set both `actor` and `container` in `runtime`"
				);
			}
			(None, None) => {
				bail_with!(
					API_BAD_BODY,
					reason = "must set one of `actor` or `container` in `runtime`"
				);
			}
		}
	}
}

impl ApiFrom<models::BuildsNetworkMode> for BuildNetworkMode {
	fn api_from(value: models::BuildsNetworkMode) -> BuildNetworkMode {
		match value {
			models::BuildsNetworkMode::Bridge => BuildNetworkMode::Bridge,
			models::BuildsNetworkMode::Host => BuildNetworkMode::Host,
		}
	}
}

impl ApiFrom<BuildNetworkMode> for models::BuildsNetworkMode {
	fn api_from(value: BuildNetworkMode) -> models::BuildsNetworkMode {
		match value {
			BuildNetworkMode::Bridge => models::BuildsNetworkMode::Bridge,
			BuildNetworkMode::Host => models::BuildsNetworkMode::Host,
		}
	}
}

impl ApiTryFrom<BuildPort> for models::BuildsPortRequest {
	type Error = GlobalError;

	fn api_try_from(value: BuildPort) -> GlobalResult<models::BuildsPortRequest> {
		Ok(models::BuildsPortRequest {
			internal_port: value.internal_port.map(TryInto::try_into).transpose()?,
			protocol: match value.routing {
				PortRouting::GameGuard { protocol } => protocol.api_into(),
				PortRouting::Host { protocol } => protocol.api_into(),
			},
			routing: match value.routing {
				PortRouting::GameGuard { .. } => Some(Box::new(models::BuildsPortRouting {
					guard: Some(json!({})),
					host: None,
				})),
				PortRouting::Host { .. } => Some(Box::new(models::BuildsPortRouting {
					guard: None,
					host: Some(json!({})),
				})),
			},
		})
	}
}

pub fn convert_port_request_from_api(
	port_name: &str,
	port: models::BuildsPortRequest,
) -> GlobalResult<BuildPort> {
	Ok(BuildPort {
		internal_port: port.internal_port.map(TryInto::try_into).transpose()?,
		routing: if let Some(routing) = port.routing {
			match *routing {
				models::BuildsPortRouting {
					guard: Some(_gg),
					host: None,
				} => PortRouting::GameGuard {
					protocol: port.protocol.api_into(),
				},
				models::BuildsPortRouting {
					guard: None,
					host: Some(_),
				} => PortRouting::Host {
					protocol: match port.protocol.api_try_into() {
						Err(err)
							if GlobalError::is(
								&err,
								formatted_error::code::BUILD_INVALID,
							) =>
						{
							// Add location
							bail_with!(
								BUILD_INVALID,
								reason = format!("network.ports[{port_name:?}].protocol: Host port protocol must be either TCP or UDP.")
							);
						}
						x => x?,
					},
				},
				models::BuildsPortRouting { .. } => {
					bail_with!(
						BUILD_INVALID,
						reason = format!("network.ports[{port_name:?}].routing: Must specify either `guard` or `host` routing type.")
					);
				}
			}
		} else {
			PortRouting::GameGuard {
				protocol: port.protocol.api_into(),
			}
		},
	})
}

impl ApiFrom<models::BuildsPortProtocol> for GameGuardProtocol {
	fn api_from(value: models::BuildsPortProtocol) -> GameGuardProtocol {
		match value {
			models::BuildsPortProtocol::Udp => GameGuardProtocol::Udp,
			models::BuildsPortProtocol::Tcp => GameGuardProtocol::Tcp,
			models::BuildsPortProtocol::Http => GameGuardProtocol::Http,
			models::BuildsPortProtocol::Https => GameGuardProtocol::Https,
			models::BuildsPortProtocol::TcpTls => GameGuardProtocol::TcpTls,
		}
	}
}

impl ApiFrom<GameGuardProtocol> for models::BuildsPortProtocol {
	fn api_from(value: GameGuardProtocol) -> models::BuildsPortProtocol {
		match value {
			GameGuardProtocol::Udp => models::BuildsPortProtocol::Udp,
			GameGuardProtocol::Tcp => models::BuildsPortProtocol::Tcp,
			GameGuardProtocol::Http => models::BuildsPortProtocol::Http,
			GameGuardProtocol::Https => models::BuildsPortProtocol::Https,
			GameGuardProtocol::TcpTls => models::BuildsPortProtocol::TcpTls,
		}
	}
}

impl ApiTryFrom<models::BuildsPortProtocol> for HostProtocol {
	type Error = GlobalError;
	fn api_try_from(value: models::BuildsPortProtocol) -> GlobalResult<HostProtocol> {
		Ok(match value {
			models::BuildsPortProtocol::Udp => HostProtocol::Udp,
			models::BuildsPortProtocol::Tcp => HostProtocol::Tcp,
			_ => {
				bail_with!(
					BUILD_INVALID,
					reason = "Host port protocol must be either TCP or UDP."
				);
			}
		})
	}
}

impl ApiFrom<HostProtocol> for models::BuildsPortProtocol {
	fn api_from(value: HostProtocol) -> models::BuildsPortProtocol {
		match value {
			HostProtocol::Udp => models::BuildsPortProtocol::Udp,
			HostProtocol::Tcp => models::BuildsPortProtocol::Tcp,
		}
	}
}

impl ApiTryFrom<models::BuildsResources> for BuildResources {
	type Error = GlobalError;

	fn api_try_from(value: models::BuildsResources) -> GlobalResult<Self> {
		ensure_with!(
			value.cpu >= 0,
			API_BAD_BODY,
			reason = "`resources.cpu` must be positive"
		);
		ensure_with!(
			value.memory >= 0,
			API_BAD_BODY,
			reason = "`resources.memory` must be positive"
		);

		Ok(BuildResources {
			cpu_millicores: value.cpu.try_into()?,
			memory_mib: value.memory.try_into()?,
		})
	}
}

impl ApiTryFrom<BuildResources> for models::BuildsResources {
	type Error = GlobalError;

	fn api_try_from(value: BuildResources) -> GlobalResult<Self> {
		Ok(models::BuildsResources {
			cpu: value.cpu_millicores.try_into()?,
			memory: value.memory_mib.try_into()?,
		})
	}
}

// TODO: Move to upload pkg when its converted to new ops
pub mod upload {
	use std::convert::TryInto;

	use chirp_workflow::prelude::*;
	use rivet_api::models;
	use rivet_convert::ApiTryFrom;
	use rivet_operation::prelude::proto::backend;

	#[derive(Debug)]
	pub struct PrepareFile {
		pub path: String,
		pub mime: Option<String>,
		pub content_length: u64,
		pub multipart: bool,
	}

	impl ApiTryFrom<models::UploadPrepareFile> for PrepareFile {
		type Error = GlobalError;

		fn api_try_from(value: models::UploadPrepareFile) -> GlobalResult<Self> {
			Ok(PrepareFile {
				path: value.path,
				mime: value.content_type,
				content_length: value.content_length.try_into()?,
				multipart: false,
			})
		}
	}

	#[derive(Debug)]
	pub struct PresignedUploadRequest {
		pub path: String,
		pub url: String,
		pub part_number: u32,
		pub byte_offset: u64,
		pub content_length: u64,
	}

	impl From<backend::upload::PresignedUploadRequest> for PresignedUploadRequest {
		fn from(value: backend::upload::PresignedUploadRequest) -> Self {
			PresignedUploadRequest {
				path: value.path,
				url: value.url,
				part_number: value.part_number,
				byte_offset: value.byte_offset,
				content_length: value.content_length,
			}
		}
	}

	impl ApiTryFrom<PresignedUploadRequest> for models::UploadPresignedRequest {
		type Error = GlobalError;

		fn api_try_from(value: PresignedUploadRequest) -> GlobalResult<Self> {
			Ok(models::UploadPresignedRequest {
				path: value.path,
				url: value.url,
				byte_offset: value.byte_offset.try_into()?,
				content_length: value.content_length.try_into()?,
			})
		}
	}
}
