use build::types::{BuildAllocationType, BuildCompression, BuildKind, BuildResources};
use chirp_workflow::prelude::*;
use cluster::types::BuildDeliveryMethod;
use fdb_util::{end_of_key_range, FormalKey, SNAPSHOT};
use foundationdb::{
	self as fdb,
	options::{ConflictRangeType, StreamingMode},
};
use futures_util::TryStreamExt;
use rand::Rng;
use sqlx::Acquire;

use super::{Input, Port};
use crate::{
	keys, protocol,
	types::{ActorLifecycle, ActorResources, GameGuardProtocol, NetworkMode, Routing},
};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct ValidateInput {
	pub env_id: Uuid,
	pub tags: util::serde::HashableMap<String, String>,
	pub resources: Option<ActorResources>,
	pub image_id: Uuid,
	pub root_user_enabled: bool,
	pub args: Vec<String>,
	pub network_mode: NetworkMode,
	pub environment: util::serde::HashableMap<String, String>,
	pub network_ports: util::serde::HashableMap<String, Port>,
}

#[activity(Validate)]
pub async fn validate(ctx: &ActivityCtx, input: &ValidateInput) -> GlobalResult<Option<String>> {
	let dc_id = ctx.config().server()?.rivet.edge()?.datacenter_id;

	let (tiers, upload_res, game_config_res) = tokio::try_join!(
		async {
			let tier_res = ctx
				.op(tier::ops::list::Input {
					datacenter_ids: vec![dc_id],
					pegboard: true,
				})
				.await?;
			let tier_dc = unwrap!(tier_res.datacenters.into_iter().next());

			GlobalResult::Ok(tier_dc.tiers)
		},
		async {
			let builds_res = ctx
				.op(build::ops::get::Input {
					build_ids: vec![input.image_id],
				})
				.await?;

			let Some(build) = builds_res.builds.into_iter().next() else {
				return Ok(None);
			};

			let uploads_res = op!([ctx] upload_get {
				upload_ids: vec![build.upload_id.into()],
			})
			.await?;

			Ok(Some((
				build,
				unwrap!(uploads_res.uploads.first()).complete_ts.is_some(),
			)))
		},
		async {
			let games_res = op!([ctx] game_resolve_namespace_id {
				namespace_ids: vec![input.env_id.into()],
			})
			.await?;

			let Some(game) = games_res.games.first() else {
				return Ok(None);
			};

			let game_config_res = ctx
				.op(crate::ops::game_config::get::Input {
					game_ids: vec![unwrap!(game.game_id).into()],
				})
				.await?;

			Ok(Some(unwrap!(game_config_res.game_configs.first()).clone()))
		}
	)?;

	// TODO: Validate build belongs to env/game
	let Some((build, upload_complete)) = upload_res else {
		return Ok(Some("Build not found.".into()));
	};

	if !upload_complete {
		return Ok(Some("Build upload not complete.".into()));
	}

	let resources = match build.allocation_type {
		BuildAllocationType::None => {
			// NOTE: This should be unreachable because if an old build is encountered the old actor wf is used.
			return Ok(Some("Old builds not supported.".into()));
		}
		BuildAllocationType::Single => {
			if let Some(resources) = &input.resources {
				resources.clone()
			} else {
				return Ok(Some(
					"Actors with builds of `allocation_type` = `single` must specify `resources`."
						.into(),
				));
			}
		}
		BuildAllocationType::Multi => {
			if input.resources.is_some() {
				return Ok(Some("Cannot specify `resources` for actors with builds of `allocation_type` = `multi`.".into()));
			}

			let build_resources = unwrap!(build.resources, "multi build should have resources");

			ActorResources {
				cpu_millicores: build_resources.cpu_millicores,
				memory_mib: build_resources.memory_mib,
			}
		}
	};

	// Find any tier that has more CPU and memory than the requested resources
	let has_tier = tiers
		.iter()
		.any(|t| t.cpu_millicores >= resources.cpu_millicores && t.memory >= resources.memory_mib);

	if !has_tier {
		return Ok(Some("Too many resources allocated.".into()));
	}

	let Some(game_config) = game_config_res else {
		return Ok(Some("Environment not found.".into()));
	};

	if matches!(input.network_mode, NetworkMode::Host) && !game_config.host_networking_enabled {
		return Ok(Some("Host networking is not enabled for this game.".into()));
	}

	if input.root_user_enabled && !game_config.root_user_enabled {
		return Ok(Some(
			"Docker root user is not enabled for this game.".into(),
		));
	}

	if input.tags.len() > 8 {
		return Ok(Some("Too many tags (max 8).".into()));
	}

	for (k, v) in &input.tags {
		if k.is_empty() {
			return Ok(Some("tags[]: Tag label cannot be empty.".into()));
		}
		if k.len() > 32 {
			return Ok(Some(format!(
				"tags[{:?}]: Tag label too large (max 32 bytes).",
				util::safe_slice(k, 0, 32),
			)));
		}
		if v.is_empty() {
			return Ok(Some(format!("tags[{k:?}]: Tag value cannot be empty.",)));
		}
		if v.len() > 1024 {
			return Ok(Some(format!(
				"tags[{k:?}]: Tag value too large (max 1024 bytes)."
			)));
		}
	}

	if input.args.len() > 64 {
		return Ok(Some("Too many arguments (max 64).".into()));
	}

	for (i, arg) in input.args.iter().enumerate() {
		if arg.len() > 256 {
			return Ok(Some(format!(
				"runtime.args[{i}]: Argument too large (max 256 bytes)."
			)));
		}
	}

	if input.environment.len() > 64 {
		return Ok(Some("Too many environment variables (max 64).".into()));
	}

	for (k, v) in &input.environment {
		if k.len() > 256 {
			return Ok(Some(format!(
				"runtime.environment[{:?}]: Key too large (max 256 bytes).",
				util::safe_slice(k, 0, 256),
			)));
		}
		if v.len() > 1024 {
			return Ok(Some(format!(
				"runtime.environment[{k:?}]: Value too large (max 1024 bytes)."
			)));
		}
	}

	if input.network_ports.len() > 8 {
		return Ok(Some("Too many ports (max 8).".into()));
	}

	for (name, port) in &input.network_ports {
		if name.len() > 16 {
			return Ok(Some(format!(
				"runtime.ports[{:?}]: Port name too large (max 16 bytes).",
				util::safe_slice(name, 0, 16),
			)));
		}

		match input.network_mode {
			NetworkMode::Bridge => {
				// NOTE: Temporary validation until we implement bridge networking for isolates
				if let BuildKind::JavaScript = build.kind {
					if port.internal_port.is_some() {
						return Ok(Some(format!(
							"runtime.ports[{name:?}].internal_port: Must be null when `network.mode` = \"bridge\" and using a JS build.",
						)));
					}
				}
			}
			NetworkMode::Host => {
				if port.internal_port.is_some() {
					return Ok(Some(format!(
						"runtime.ports[{name:?}].internal_port: Must be null when `network.mode` = \"host\".",
					)));
				}
			}
		}
	}

	Ok(None)
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct DisableTlsPortsInput {
	pub network_ports: util::serde::HashableMap<String, Port>,
}

/// If TLS is not enabled in the cluster, we downgrade all protocols to the non-TLS equivalents.
/// This allows developers to develop locally with the same code they would use in production.
#[activity(DisableTlsPorts)]
pub async fn disable_tls_ports(
	ctx: &ActivityCtx,
	input: &DisableTlsPortsInput,
) -> GlobalResult<util::serde::HashableMap<String, Port>> {
	if ctx.config().server()?.rivet.guard.tls_enabled() {
		// Do nothing
		Ok(input.network_ports.clone())
	} else {
		// Downgrade all TLS protocols to non-TLS protocols
		let network_ports = input
			.network_ports
			.clone()
			.into_iter()
			.map(|(k, p)| {
				(
					k,
					Port {
						internal_port: p.internal_port,
						routing: match p.routing {
							Routing::GameGuard { protocol } => Routing::GameGuard {
								protocol: match protocol {
									GameGuardProtocol::Https => GameGuardProtocol::Http,
									GameGuardProtocol::TcpTls => GameGuardProtocol::Tcp,
									x @ (GameGuardProtocol::Http
									| GameGuardProtocol::Tcp
									| GameGuardProtocol::Udp) => x,
								},
							},
							x @ Routing::Host { .. } => x,
						},
					},
				)
			})
			.collect();

		Ok(network_ports)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct InsertDbInput {
	actor_id: util::Id,
	env_id: Uuid,
	tags: util::serde::HashableMap<String, String>,
	resources: Option<ActorResources>,
	lifecycle: ActorLifecycle,
	image_id: Uuid,
	args: Vec<String>,
	network_mode: NetworkMode,
	environment: util::serde::HashableMap<String, String>,
}

#[activity(InsertDb)]
async fn insert_db(ctx: &ActivityCtx, input: &InsertDbInput) -> GlobalResult<i64> {
	let pool = ctx.sqlite().await?;
	let create_ts = ctx.ts();

	sql_execute!(
		[ctx, &pool]
		"
		INSERT INTO state (
			env_id,
			tags,
			resources_cpu_millicores,
			resources_memory_mib,
			lifecycle_kill_timeout_ms,
			lifecycle_durable,
			create_ts,
			image_id,
			args,
			network_mode,
			environment
		)
		VALUES (?, jsonb(?), ?, ?, ?, ?, ?, ?, jsonb(?), ?, jsonb(?))
		",
		input.env_id,
		serde_json::to_string(&input.tags)?,
		input.resources.as_ref().map(|x| x.cpu_millicores as i32),
		input.resources.as_ref().map(|x| x.memory_mib as i32),
		input.lifecycle.kill_timeout_ms,
		input.lifecycle.durable,
		create_ts,
		input.image_id,
		serde_json::to_string(&input.args)?,
		input.network_mode as i32,
		serde_json::to_string(&input.environment)?,
	)
	.await?;

	Ok(create_ts)
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct AllocateIngressPortsInput {
	actor_id: util::Id,
	network_ports: util::serde::HashableMap<String, Port>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AllocateIngressPortsOutput {
	ports: Vec<(GameGuardProtocol, Vec<u16>)>,
}

#[activity(AllocateIngressPorts)]
async fn allocate_ingress_ports(
	ctx: &ActivityCtx,
	input: &AllocateIngressPortsInput,
) -> GlobalResult<AllocateIngressPortsOutput> {
	// Count up ports per protocol
	let mut port_counts = Vec::new();
	for (_, port) in &input.network_ports {
		match port.routing {
			Routing::GameGuard {
				protocol:
					protocol @ (GameGuardProtocol::Tcp
					| GameGuardProtocol::TcpTls
					| GameGuardProtocol::Udp),
			} => {
				if let Some((_, count)) = port_counts.iter_mut().find(|(p, _)| &protocol == p) {
					*count += 1;
				} else {
					port_counts.push((protocol, 1));
				}
			}
			_ => {}
		}
	}

	let gg_config = &ctx.config().server()?.rivet.guard;

	// Choose which port to assign for a job's ingress port.
	// This is required because TCP and UDP do not have a `Host` header and thus cannot be re-routed by hostname.
	//
	// If not provided by `ProxiedPort`, then:
	// - HTTP: 80
	// - HTTPS: 443
	// - TCP/TLS: random
	// - UDP: random
	let ports = ctx
		.fdb()
		.await?
		.run(|tx, _mc| {
			let port_counts = port_counts.clone();
			async move {
				let mut results = Vec::new();

				// TODO: Parallelize
				for (protocol, count) in &port_counts {
					// Determine port range per protocol
					let port_range = match protocol {
						GameGuardProtocol::Http | GameGuardProtocol::Https => {
							return Err(fdb::FdbBindingError::CustomError(
								"Dynamic allocation not implemented for http/https ports".into(),
							));
						}
						GameGuardProtocol::Tcp | GameGuardProtocol::TcpTls => {
							gg_config.min_ingress_port_tcp()..=gg_config.max_ingress_port_tcp()
						}
						GameGuardProtocol::Udp => {
							gg_config.min_ingress_port_udp()..=gg_config.max_ingress_port_udp()
						}
					};

					let mut last_port = None;
					let mut ports = Vec::new();

					// Choose a random starting port for better spread and less cache hits
					let mut start = {
						// It is important that we don't start at the end of the range so that the logic with
						// `last_port` works correctly
						let exclusive_port_range = *port_range.start()..*port_range.end();
						rand::thread_rng().gen_range(exclusive_port_range)
					};

					// Build start and end keys for ingress ports subspace
					let start_key = keys::subspace()
						.subspace(&keys::port::IngressKey2::subspace(*protocol, start))
						.range()
						.0;
					let end_key = keys::subspace()
						.subspace(&keys::port::IngressKey2::subspace(
							*protocol,
							*port_range.end(),
						))
						.range()
						.1;
					let mut stream = tx.get_ranges_keyvalues(
						fdb::RangeOption {
							mode: StreamingMode::Iterator,
							..(start_key, end_key.clone()).into()
						},
						// NOTE: This is not SERIALIZABLE because we don't want to conflict with all of the keys,
						// just the one we choose
						SNAPSHOT,
					);

					// Continue iterating over the same stream until all of the required ports are found
					for _ in 0..*count {
						// Iterate through the subspace range until a port is found
						let port = loop {
							let Some(entry) = stream.try_next().await? else {
								match last_port {
									Some(port) if port == *port_range.end() => {
										// End of range reached, start a new range read from the beginning (wrap around)
										if start != *port_range.start() {
											last_port = None;

											let old_start = start;
											start = *port_range.start();

											let start_key = keys::subspace()
												.subspace(&keys::port::IngressKey2::subspace(
													*protocol, start,
												))
												.range()
												.0;
											stream = tx.get_ranges_keyvalues(
												fdb::RangeOption {
													mode: StreamingMode::Iterator,
													limit: Some(old_start as usize),
													..(start_key, end_key.clone()).into()
												},
												// NOTE: This is not SERIALIZABLE because we don't want to conflict
												// with all of the keys, just the one we choose
												SNAPSHOT,
											);

											continue;
										} else {
											break None;
										}
									}
									// Return port after last port
									Some(last_port) => {
										break Some(last_port + 1);
									}
									// No ports were returned (range is empty)
									None => {
										break Some(start);
									}
								}
							};

							let key = keys::subspace()
								.unpack::<keys::port::IngressKey2>(entry.key())
								.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
							let current_port = key.port;

							if let Some(last_port) = last_port {
								// Gap found
								if current_port != last_port + 1 {
									break Some(last_port + 1);
								}
							}

							last_port = Some(current_port);
						};

						let Some(port) = port else {
							return Err(fdb::FdbBindingError::CustomError(
								format!("not enough {protocol} ports available").into(),
							));
						};

						let ingress_port_key =
							keys::port::IngressKey2::new(*protocol, port, input.actor_id);
						let ingress_port_key_buf = keys::subspace().pack(&ingress_port_key);

						// Add read conflict only for this key
						tx.add_conflict_range(
							&ingress_port_key_buf,
							&end_of_key_range(&ingress_port_key_buf),
							ConflictRangeType::Read,
						)?;

						// Set key
						tx.set(
							&ingress_port_key_buf,
							&ingress_port_key
								.serialize(())
								.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
						);

						ports.push(port);
					}

					results.push((*protocol, ports));
				}

				Ok(results)
			}
		})
		.custom_instrument(tracing::info_span!("allocate_ingress_ports_tx"))
		.await?;

	Ok(AllocateIngressPortsOutput { ports })
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct InsertPortsInput {
	actor_id: util::Id,
	network_ports: util::serde::HashableMap<String, Port>,
	ingress_ports: Vec<(GameGuardProtocol, Vec<u16>)>,
}

#[activity(InsertPorts)]
async fn insert_ports(ctx: &ActivityCtx, input: &InsertPortsInput) -> GlobalResult<()> {
	let pool = ctx.sqlite().await?;
	let mut conn = pool.conn().await?;
	let mut tx = conn.begin().await?;

	let gg_config = &ctx.config().server()?.rivet.guard;
	let mut ingress_ports = input
		.ingress_ports
		.iter()
		.map(|(protocol, ports)| (protocol, ports.clone().into_iter()))
		.collect::<Vec<_>>();

	for (name, port) in input.network_ports.iter() {
		match port.routing {
			Routing::GameGuard { protocol } => {
				sql_execute!(
					[ctx, @tx &mut tx]
					"
					INSERT INTO ports_ingress (
						port_name,
						port_number,
						protocol,
						ingress_port_number
					)
					VALUES (?, ?, ?, ?)
					",
					name,
					port.internal_port.map(|x| x as i32),
					protocol as i32,
					match protocol {
						GameGuardProtocol::Http => gg_config.http_port(),
						GameGuardProtocol::Https => gg_config.https_port(),
						GameGuardProtocol::Tcp | GameGuardProtocol::TcpTls | GameGuardProtocol::Udp => {
							let (_, ports_iter) = unwrap!(
								ingress_ports.iter_mut().find(|(p, _)| &&protocol == p)
							);
							unwrap!(ports_iter.next(), "missing ingress port")
						},
					} as i32,
				)
				.await?;
			}
			Routing::Host { protocol } => {
				sql_execute!(
					[ctx, @tx &mut tx]
					"
					INSERT INTO ports_host (
						port_name,
						port_number,
						protocol
					)
					VALUES (?, ?, ?)
					",
					name,
					port.internal_port.map(|x| x as i32),
					protocol as i32,
				)
				.await?;
			}
		};
	}

	tx.commit().await?;

	Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct InsertFdbInput {
	actor_id: util::Id,
	env_id: Uuid,
	tags: util::serde::HashableMap<String, String>,
	create_ts: i64,
}

#[activity(InsertFdb)]
async fn insert_fdb(ctx: &ActivityCtx, input: &InsertFdbInput) -> GlobalResult<()> {
	ctx.fdb()
		.await?
		.run(|tx, _mc| async move {
			let create_ts_key = keys::actor2::CreateTsKey::new(input.actor_id);
			tx.set(
				&keys::subspace().pack(&create_ts_key),
				&create_ts_key
					.serialize(input.create_ts)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			);

			let workflow_id_key = keys::actor2::WorkflowIdKey::new(input.actor_id);
			tx.set(
				&keys::subspace().pack(&workflow_id_key),
				&workflow_id_key
					.serialize(ctx.workflow_id())
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			);

			// Add env index key
			let env_actor_key =
				keys::env::Actor2Key::new(input.env_id, input.create_ts, input.actor_id);
			let data = keys::env::Actor2KeyData {
				is_destroyed: false,
				tags: input.tags.clone().into_iter().collect(),
			};
			tx.set(
				&keys::subspace().pack(&env_actor_key),
				&env_actor_key
					.serialize(data)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			);

			Ok(())
		})
		.custom_instrument(tracing::info_span!("actor_insert_tx"))
		.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct GetMetaInput {
	env_id: Uuid,
	image_id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct GetMetaOutput {
	pub project_id: Uuid,
	pub project_slug: String,
	pub env_slug: String,
	pub build_upload_id: Uuid,
	pub build_file_name: String,
	pub build_kind: BuildKind,
	pub build_compression: BuildCompression,
	pub build_allocation_type: BuildAllocationType,
	pub build_allocation_total_slots: u64,
	pub build_resources: Option<BuildResources>,
	pub dc_name_id: String,
	pub dc_display_name: String,
	pub dc_build_delivery_method: BuildDeliveryMethod,
}

#[activity(GetMeta)]
async fn get_meta(ctx: &ActivityCtx, input: &GetMetaInput) -> GlobalResult<GetMetaOutput> {
	let dc_id = ctx.config().server()?.rivet.edge()?.datacenter_id;

	let (env_res, build_res, dc_res) = tokio::try_join!(
		op!([ctx] game_namespace_get {
			namespace_ids: vec![input.env_id.into()],
		}),
		ctx.op(build::ops::get::Input {
			build_ids: vec![input.image_id],
		}),
		ctx.op(cluster::ops::datacenter::get::Input {
			datacenter_ids: vec![dc_id],
		})
	)?;
	let env = unwrap_with!(env_res.namespaces.first(), ENVIRONMENT_NOT_FOUND);
	let build = unwrap_with!(build_res.builds.first(), BUILD_NOT_FOUND);
	let dc = unwrap!(dc_res.datacenters.first());

	// Lookup project
	let project_id = unwrap!(env.game_id).as_uuid();
	let projects_res = op!([ctx] game_get {
		game_ids: vec![project_id.into()],
	})
	.await?;
	let project = unwrap!(projects_res.games.first());

	Ok(GetMetaOutput {
		project_id,
		project_slug: project.name_id.clone(),
		env_slug: env.name_id.clone(),
		build_upload_id: build.upload_id,
		build_file_name: build::utils::file_name(build.kind, build.compression),
		build_kind: build.kind,
		build_compression: build.compression,
		build_allocation_type: build.allocation_type,
		build_allocation_total_slots: build.allocation_total_slots,
		build_resources: build.resources.clone(),
		dc_name_id: dc.name_id.clone(),
		dc_display_name: dc.display_name.clone(),
		dc_build_delivery_method: dc.build_delivery_method,
	})
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct InsertMetaInput {
	project_id: Uuid,
	build_kind: BuildKind,
	build_compression: BuildCompression,
	root_user_enabled: bool,
}

#[activity(InsertMeta)]
async fn insert_meta(ctx: &ActivityCtx, input: &InsertMetaInput) -> GlobalResult<()> {
	let pool = ctx.sqlite().await?;

	sql_execute!(
		[ctx, pool]
		"
		UPDATE state
		SET
			project_id = ?,
			build_kind = ?,
			build_compression = ?,
			root_user_enabled = ?
		",
		input.project_id,
		input.build_kind as i64,
		input.build_compression as i64,
		input.root_user_enabled,
	)
	.await?;

	Ok(())
}

pub enum SetupCtx {
	Init {
		network_ports: util::serde::HashableMap<String, Port>,
	},
	Reschedule {
		image_id: Uuid,
	},
}

#[derive(Clone)]
pub struct ActorSetupCtx {
	pub image_id: Uuid,
	pub meta: GetMetaOutput,
	pub resources: protocol::Resources,
	pub artifact_url_stub: String,
	pub fallback_artifact_url: String,
	/// Bytes.
	pub artifact_size: u64,
}

pub async fn setup(
	ctx: &mut WorkflowCtx,
	input: &Input,
	setup: SetupCtx,
) -> GlobalResult<ActorSetupCtx> {
	let image_id = match setup {
		SetupCtx::Init { network_ports } => {
			let tags = input.tags.clone();
			let create_ts = ctx
				.activity(InsertDbInput {
					actor_id: input.actor_id,
					env_id: input.env_id,
					tags: tags.clone(),
					resources: input.resources.clone(),
					lifecycle: input.lifecycle.clone(),
					image_id: input.image_id,
					args: input.args.clone(),
					network_mode: input.network_mode,
					environment: input.environment.clone(),
				})
				.await?;

			let ingress_ports_res = ctx
				.activity(AllocateIngressPortsInput {
					actor_id: input.actor_id,
					network_ports: network_ports.clone(),
				})
				.await?;

			ctx.activity(InsertPortsInput {
				actor_id: input.actor_id,
				network_ports,
				ingress_ports: ingress_ports_res.ports,
			})
			.await?;

			ctx.activity(InsertFdbInput {
				actor_id: input.actor_id,
				env_id: input.env_id,
				tags,
				create_ts,
			})
			.await?;

			input.image_id
		}
		SetupCtx::Reschedule { image_id } => image_id,
	};

	let meta = ctx
		.activity(GetMetaInput {
			env_id: input.env_id,
			image_id,
		})
		.await?;

	ctx.v(2)
		.activity(InsertMetaInput {
			project_id: meta.project_id,
			build_kind: meta.build_kind,
			build_compression: meta.build_compression,
			root_user_enabled: input.root_user_enabled,
		})
		.await?;

	// Use resources from build or from actor config
	let resources = match meta.build_allocation_type {
		BuildAllocationType::None => bail!("actors do not support old builds"),
		BuildAllocationType::Single => unwrap!(
			input.resources.clone(),
			"single builds should have actor resources"
		),
		BuildAllocationType::Multi => {
			let build_resources =
				unwrap_ref!(meta.build_resources, "multi builds should have resources");

			ActorResources {
				cpu_millicores: build_resources.cpu_millicores,
				memory_mib: build_resources.memory_mib,
			}
		}
	};

	let (resources, artifacts_res) = ctx
		.join((
			activity(SelectResourcesInput {
				cpu_millicores: resources.cpu_millicores,
				memory_mib: resources.memory_mib,
			}),
			activity(ResolveArtifactsInput {
				build_upload_id: meta.build_upload_id,
				build_file_name: meta.build_file_name.clone(),
				dc_build_delivery_method: meta.dc_build_delivery_method,
			}),
		))
		.await?;

	Ok(ActorSetupCtx {
		image_id,
		meta,
		resources,
		artifact_url_stub: artifacts_res.artifact_url_stub,
		fallback_artifact_url: artifacts_res.fallback_artifact_url,
		artifact_size: artifacts_res.artifact_size,
	})
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct SelectResourcesInput {
	cpu_millicores: u32,
	memory_mib: u32,
}

#[activity(SelectResources)]
async fn select_resources(
	ctx: &ActivityCtx,
	input: &SelectResourcesInput,
) -> GlobalResult<protocol::Resources> {
	let dc_id = ctx.config().server()?.rivet.edge()?.datacenter_id;

	let tier_res = ctx
		.op(tier::ops::list::Input {
			datacenter_ids: vec![dc_id],
			pegboard: true,
		})
		.await?;
	let tier_dc = unwrap!(tier_res.datacenters.first());
	let mut tiers = tier_dc.tiers.iter().collect::<Vec<_>>();

	// Sort the tiers by cpu
	tiers.sort_by(|a, b| a.cpu.cmp(&b.cpu));

	// Find the first tier that has more CPU and memory than the requested
	// resources
	let tier = unwrap!(
		tiers
			.iter()
			.find(|t| { t.cpu_millicores >= input.cpu_millicores && t.memory >= input.memory_mib }),
		"no suitable tier found"
	);

	// runc-compatible resources
	let cpu = tier.rivet_cores_numerator as u64 * 1_000 / tier.rivet_cores_denominator as u64; // Millicore (1/1000 of a core)
	let memory = tier.memory as u64 * (1024 * 1024);
	let memory_max = tier.memory_max as u64 * (1024 * 1024);

	let pool = ctx.sqlite().await?;

	// Write to db
	sql_execute!(
		[ctx, pool]
		"
		UPDATE state
		SET
			selected_resources_cpu_millicores = ?,
			selected_resources_memory_mib = ?
		",
		i64::try_from(cpu)?,
		i64::try_from(tier.memory)?,
	)
	.await?;

	Ok(protocol::Resources {
		cpu,
		memory,
		memory_max,
		disk: tier.disk,
	})
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct ResolveArtifactsInput {
	build_upload_id: Uuid,
	build_file_name: String,
	dc_build_delivery_method: BuildDeliveryMethod,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct ResolveArtifactsOutput {
	artifact_url_stub: String,
	fallback_artifact_url: String,
	/// Bytes.
	artifact_size: u64,
}

#[activity(ResolveArtifacts)]
async fn resolve_artifacts(
	ctx: &ActivityCtx,
	input: &ResolveArtifactsInput,
) -> GlobalResult<ResolveArtifactsOutput> {
	// Get the fallback URL
	let fallback_artifact_url = {
		tracing::debug!("using s3 direct delivery");

		// Build client
		let s3_client = s3_util::Client::with_bucket_and_endpoint(
			ctx.config(),
			"bucket-build",
			s3_util::EndpointKind::EdgeInternal,
		)
		.await?;

		let presigned_req = s3_client
			.get_object()
			.bucket(s3_client.bucket())
			.key(format!(
				"{upload_id}/{file_name}",
				upload_id = input.build_upload_id,
				file_name = input.build_file_name,
			))
			.presigned(
				s3_util::aws_sdk_s3::presigning::PresigningConfig::builder()
					.expires_in(std::time::Duration::from_secs(15 * 60))
					.build()?,
			)
			.await?;

		let addr_str = presigned_req.uri().to_string();
		tracing::debug!(addr = %addr_str, "resolved artifact s3 presigned request");

		addr_str
	};

	// Get the artifact size
	let uploads_res = op!([ctx] upload_get {
		upload_ids: vec![input.build_upload_id.into()],
	})
	.await?;
	let upload = unwrap!(uploads_res.uploads.first());

	Ok(ResolveArtifactsOutput {
		artifact_url_stub: crate::util::image_artifact_url_stub(
			ctx.config(),
			input.build_upload_id,
			&input.build_file_name,
		)?,
		fallback_artifact_url,
		artifact_size: upload.content_length,
	})
}
