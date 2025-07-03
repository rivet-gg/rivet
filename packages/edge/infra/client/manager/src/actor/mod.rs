use std::{
	collections::HashMap,
	result::Result::{Err, Ok},
	sync::Arc,
};

use anyhow::*;
use indoc::indoc;
use nix::{sys::signal::Signal, unistd::Pid};
use pegboard::protocol;
use pegboard_actor_kv as kv;
use pegboard_config::runner_protocol;

use crate::{ctx::Ctx, runner, utils};

pub struct Actor {
	actor_id: rivet_util::Id,
	generation: u32,
	config: protocol::ActorConfig,
	runner: Arc<runner::Runner>,
	kv: kv::ActorKv,
}

impl Actor {
	pub fn new(
		fdb: &utils::fdb::FdbPool,
		actor_id: rivet_util::Id,
		generation: u32,
		config: protocol::ActorConfig,
		runner: Arc<runner::Runner>,
	) -> Arc<Self> {
		Arc::new(Actor {
			actor_id,
			generation,
			config,
			runner,
			kv: kv::ActorKv::new((&**fdb).clone(), actor_id),
		})
	}

	pub async fn start(self: &Arc<Self>, ctx: &Arc<Ctx>) -> Result<()> {
		tracing::info!(actor_id=?self.actor_id, generation=?self.generation, "starting");

		let runner_id = self
			.config
			.runner
			.as_ref()
			.context("should have runner")?
			.runner_id();
		let config_json = serde_json::to_vec(&self.config)?;

		// Write actor to DB
		utils::sql::query(|| async {
			// NOTE: On conflict here in case this query runs but the command is not acknowledged
			sqlx::query(indoc!(
				"
				INSERT INTO actors (
					actor_id,
					generation,
					runner_id,
					config,
					start_ts
				)
				VALUES (?1, ?2, ?3, ?4, ?5)
				ON CONFLICT (actor_id, generation) DO NOTHING
				",
			))
			.bind(self.actor_id)
			.bind(self.generation as i64)
			.bind(runner_id)
			.bind(&config_json)
			.bind(utils::now())
			.execute(&mut *ctx.sql().await?)
			.await
		})
		.await?;

		ctx.event(protocol::Event::ActorStateUpdate {
			actor_id: self.actor_id,
			generation: self.generation,
			state: protocol::ActorState::Starting,
		})
		.await?;

		// Lifecycle
		let self2 = self.clone();
		let ctx2 = ctx.clone();
		tokio::spawn(async move {
			match self2.run(&ctx2).await {
				Ok(observer) => {
					if let Err(err) = self2.observe(&ctx2, observer).await {
						tracing::error!(actor_id=?self2.actor_id, ?err, "observe failed");
					}
				}
				Err(err) => {
					tracing::error!(actor_id=?self2.actor_id, ?err, "run failed")
				}
			}

			// Cleanup afterwards
			if let Err(err) = self2.cleanup(&ctx2).await {
				tracing::error!(actor_id=?self2.actor_id, ?err, "cleanup failed");
			}
		});

		Ok(())
	}

	async fn run(self: &Arc<Self>, ctx: &Arc<Ctx>) -> Result<runner::ActorProxy> {
		tracing::info!(actor_id=?self.actor_id, generation=?self.generation, "running");

		// NOTE: Create actor proxy before sending the start actor message to prevent a race
		// condition
		let actor_proxy = self.runner.new_actor_proxy(self.actor_id, self.generation);

		match self
			.config
			.runner
			.as_ref()
			.context("should have runner config")?
		{
			protocol::ActorRunner::New { .. } => {
				// Because the runner is not already started we can get the ports here instead of reading from
				// sqlite
				let ports = self.runner.start(ctx).await?;

				let pid = self.runner.pid().await?;

				tracing::info!(actor_id=?self.actor_id, generation=?self.generation, ?pid, "pid received");

				match self.runner.config().image.allocation_type {
					protocol::ImageAllocationType::Single => {
						self.set_running(ctx, pid, ports).await?
					}
					protocol::ImageAllocationType::Multi => {
						self.runner
							.send(&runner_protocol::proto::ToRunner {
								message: Some(
									runner_protocol::proto::to_runner::Message::StartActor(
										runner_protocol::proto::to_runner::StartActor {
											actor_id: self.actor_id.to_string(),
											generation: self.generation,
											env: self.runner.config().env.clone().into(),
											metadata: Some(convert_actor_metadata_to_proto(
												&self.config.metadata.deserialize()?,
											)),
										},
									),
								),
							})
							.await?;
					}
				};
			}
			protocol::ActorRunner::Existing { .. } => {
				match self.runner.config().image.allocation_type {
					protocol::ImageAllocationType::Single => {
						unimplemented!(
							"allocating new actor to an existing `Single` allocation_type runner"
						);
					}
					protocol::ImageAllocationType::Multi => {
						self.runner
							.send(&runner_protocol::proto::ToRunner {
								message: Some(
									runner_protocol::proto::to_runner::Message::StartActor(
										runner_protocol::proto::to_runner::StartActor {
											actor_id: self.actor_id.to_string(),
											generation: self.generation,
											env: self.runner.config().env.clone().into(),
											metadata: Some(convert_actor_metadata_to_proto(
												&self.config.metadata.deserialize()?,
											)),
										},
									),
								),
							})
							.await?;
					}
				};
			}
		}

		Ok(actor_proxy)
	}

	// Watch actor for updates
	pub(crate) async fn observe(
		&self,
		ctx: &Arc<Ctx>,
		mut actor_proxy: runner::ActorProxy,
	) -> Result<()> {
		tracing::info!(actor_id=?self.actor_id, generation=?self.generation, "observing");

		let exit_code = loop {
			tokio::select! {
				// We have to check if the shared runner exited or if the actor exited
				res = self.runner.observe(ctx, true) => break res?,
				res = actor_proxy.next() => {
					let Some(res) = res else {
						// Channel closed
						break None;
					};

					match res {
						runner_protocol::ToActor::StateUpdate { state } => {
							match state.state.context("ActorState.state")? {
								runner_protocol::proto::actor_state::State::Running(_) => {
									tracing::info!(
										actor_id=?self.actor_id,
										generation=?self.generation,
										"actor set to running"
									);

									let (pid, ports) = tokio::try_join!(
										self.runner.pid(),
										self.runner.ports(ctx),
									)?;

									self.set_running(ctx, pid, ports).await?;
								},
								runner_protocol::proto::actor_state::State::Exited(state) => {
									break state.exit_code;
								}
							}
						}
						runner_protocol::ToActor::Kv(req) => {
							// TODO: Add queue and bg thread for processing kv ops
							// Run kv operation
							match req.data.context("Request.data")? {
								runner_protocol::proto::kv::request::Data::Get(body) => {
									let res = self.kv.get(pegboard_actor_kv::Key::convert_vec(body.keys)).await;

									self.runner.send(&runner_protocol::proto::ToRunner {
										message: Some(runner_protocol::proto::to_runner::Message::Kv(
											runner_protocol::proto::kv::Response {
												request_id: req.request_id,
												data: match res {
													Ok(entries) => {
														let (keys, values) = entries
															.into_iter()
															.map(|(k, v)| (k.into(), v.into()))
															.unzip();
														Some(runner_protocol::proto::kv::response::Data::Get(
															runner_protocol::proto::kv::response::Get {
																keys,
																values,
															}
														))
													}
													Err(err) => {
														Some(runner_protocol::proto::kv::response::Data::Error(
															runner_protocol::proto::kv::response::Error {
																message: err.to_string(),
															}
														))
													}
												},
											}
										)),
									}).await?;
								}
								runner_protocol::proto::kv::request::Data::List(body) => {
									let res = self.kv.list(
										body.query.context("List.query")?.try_into()?,
										body.reverse,
										body.limit.map(TryInto::try_into).transpose()?
									).await;

									self.runner.send(&runner_protocol::proto::ToRunner {
										message: Some(runner_protocol::proto::to_runner::Message::Kv(
											runner_protocol::proto::kv::Response {
												request_id: req.request_id,
												data: match res {
													Ok(entries) => {
														let (keys, values) = entries
															.into_iter()
															.map(|(k, v)| (k.into(), v.into()))
															.unzip();
														Some(runner_protocol::proto::kv::response::Data::List(
															runner_protocol::proto::kv::response::List {
																keys,
																values,
															}
														))
													}
													Err(err) => {
														Some(runner_protocol::proto::kv::response::Data::Error(
															runner_protocol::proto::kv::response::Error {
																message: err.to_string(),
															}
														))
													}
												},
											}
										)),
									}).await?;
								}
								runner_protocol::proto::kv::request::Data::Put(body) => {
									let res = self.kv.put(
										body.keys
											.into_iter()
											.map(|x| x.into())
											.zip(body.values.into_iter().map(|x| x.into()))
											.collect()
									).await;

									self.runner.send(&runner_protocol::proto::ToRunner {
										message: Some(runner_protocol::proto::to_runner::Message::Kv(
											runner_protocol::proto::kv::Response {
												request_id: req.request_id,
												data: match res {
													Ok(_) => {
														Some(runner_protocol::proto::kv::response::Data::Put(
															runner_protocol::proto::kv::response::Put {}
														))
													}
													Err(err) => {
														Some(runner_protocol::proto::kv::response::Data::Error(
															runner_protocol::proto::kv::response::Error {
																message: err.to_string(),
															}
														))
													}
												},
											}
										)),
									}).await?;
								}
								runner_protocol::proto::kv::request::Data::Delete(body) => {
									let res = self.kv.delete(pegboard_actor_kv::Key::convert_vec(body.keys)).await;

									self.runner.send(&runner_protocol::proto::ToRunner {
										message: Some(runner_protocol::proto::to_runner::Message::Kv(
											runner_protocol::proto::kv::Response {
												request_id: req.request_id,
												data: match res {
													Ok(_) => {
														Some(runner_protocol::proto::kv::response::Data::Delete(
															runner_protocol::proto::kv::response::Delete {}
														))
													}
													Err(err) => {
														Some(runner_protocol::proto::kv::response::Data::Error(
															runner_protocol::proto::kv::response::Error {
																message: err.to_string(),
															}
														))
													}
												},
											}
										)),
									}).await?;
								}
								runner_protocol::proto::kv::request::Data::Drop(_) => {
									let res = self.kv.delete_all().await;

									self.runner.send(&runner_protocol::proto::ToRunner {
										message: Some(runner_protocol::proto::to_runner::Message::Kv(
											runner_protocol::proto::kv::Response {
												request_id: req.request_id,
												data: match res {
													Ok(_) => {
														Some(runner_protocol::proto::kv::response::Data::Drop(
															runner_protocol::proto::kv::response::Drop {}
														))
													}
													Err(err) => {
														Some(runner_protocol::proto::kv::response::Data::Error(
															runner_protocol::proto::kv::response::Error {
																message: err.to_string(),
															}
														))
													}
												},
											}
										)),
									}).await?;
								}
							}
						}
					}
				}
			}
		};

		self.set_exit_code(ctx, exit_code).await?;

		tracing::info!(actor_id=?self.actor_id, generation=?self.generation, "complete");

		Ok(())
	}

	pub async fn signal(
		self: &Arc<Self>,
		ctx: &Arc<Ctx>,
		signal: Signal,
		persist_storage: bool,
	) -> Result<()> {
		tracing::info!(actor_id=?self.actor_id, generation=?self.generation, ?signal, "sending signal");

		let self2 = self.clone();
		let ctx2 = ctx.clone();
		tokio::spawn(async move {
			if let Err(err) = self2.signal_inner(&ctx2, signal, persist_storage).await {
				tracing::error!(?err, "actor signal failed");
			}
		});

		Ok(())
	}

	async fn signal_inner(
		self: &Arc<Self>,
		ctx: &Arc<Ctx>,
		signal: Signal,
		persist_storage: bool,
	) -> Result<()> {
		let has_pid = self.runner.pid().await.is_ok();

		if has_pid {
			// Send message
			if self.runner.has_socket() {
				self.runner
					.send(&runner_protocol::proto::ToRunner {
						message: Some(runner_protocol::proto::to_runner::Message::SignalActor(
							runner_protocol::proto::to_runner::SignalActor {
								actor_id: self.actor_id.to_string(),
								generation: self.generation,
								signal: signal as i32,
								persist_storage,
							},
						)),
					})
					.await?;
			}
			// Send signal
			else {
				self.runner.signal(ctx, signal).await?;
			}
		}

		// Update stop_ts
		if matches!(signal, Signal::SIGTERM | Signal::SIGKILL) || !has_pid {
			let stop_ts_set = utils::sql::query(|| async {
				sqlx::query_as::<_, (bool,)>(indoc!(
					"
					UPDATE actors
					SET stop_ts = ?3
					WHERE
						actor_id = ?1 AND
						generation = ?2 AND
						stop_ts IS NULL
					RETURNING 1
					",
				))
				.bind(self.actor_id)
				.bind(self.generation as i64)
				.bind(utils::now())
				.fetch_optional(&mut *ctx.sql().await?)
				.await
			})
			.await?
			.is_some();

			// Emit event if not stopped before
			if stop_ts_set {
				ctx.event(protocol::Event::ActorStateUpdate {
					actor_id: self.actor_id,
					generation: self.generation,
					state: protocol::ActorState::Stopped,
				})
				.await?;
			}
		}

		Ok(())
	}

	#[tracing::instrument(skip_all)]
	async fn set_running(
		&self,
		ctx: &Ctx,
		pid: Pid,
		ports: protocol::HashableMap<String, protocol::ProxiedPort>,
	) -> Result<()> {
		// Update DB
		utils::sql::query(|| async {
			sqlx::query(indoc!(
				"
				UPDATE actors
				SET running_ts = ?3
				WHERE
					actor_id = ?1 AND
					generation = ?2
				",
			))
			.bind(self.actor_id)
			.bind(self.generation as i64)
			.bind(utils::now())
			.execute(&mut *ctx.sql().await?)
			.await
		})
		.await?;

		ctx.event(protocol::Event::ActorStateUpdate {
			actor_id: self.actor_id,
			generation: self.generation,
			state: protocol::ActorState::Running {
				pid: pid.as_raw().try_into()?,
				ports,
			},
		})
		.await?;

		Ok(())
	}

	#[tracing::instrument(skip_all)]
	pub async fn set_exit_code(&self, ctx: &Ctx, exit_code: Option<i32>) -> Result<()> {
		// Update DB
		let row = utils::sql::query(|| async {
			sqlx::query_as::<_, (bool,)>(indoc!(
				"
				UPDATE actors
				SET
					exit_ts = ?3,
					exit_code = ?4
				WHERE
					actor_id = ?1 AND
					generation = ?2 AND
					exit_ts IS NULL
				RETURNING 1
				",
			))
			.bind(self.actor_id)
			.bind(self.generation as i64)
			.bind(utils::now())
			.bind(exit_code)
			.fetch_optional(&mut *ctx.sql().await?)
			.await
		})
		.await?;

		// Already exited
		if row.is_none() {
			return Ok(());
		}

		ctx.event(protocol::Event::ActorStateUpdate {
			actor_id: self.actor_id,
			generation: self.generation,
			state: protocol::ActorState::Exited { exit_code },
		})
		.await?;

		Ok(())
	}

	#[tracing::instrument(skip_all)]
	pub async fn cleanup(&self, ctx: &Ctx) -> Result<()> {
		tracing::info!(actor_id=?self.actor_id, generation=?self.generation, "cleaning up actor");

		// Set exit code if it hasn't already been set
		self.set_exit_code(ctx, None).await?;

		// It is important that we remove from the actors list last so that we prevent duplicates
		{
			let mut actors = ctx.actors.write().await;
			actors.remove(&(self.actor_id, self.generation));
		}

		Ok(())
	}
}

/// Convert from serde ActorMetadata to proto ActorMetadata
pub fn convert_actor_metadata_to_proto(
	metadata: &protocol::ActorMetadata,
) -> runner_protocol::proto::ActorMetadata {
	use runner_protocol::proto::{
		actor_metadata, routing, ActorMetadata, GameGuardProtocol, HostProtocol, Port, Routing,
	};

	let mut proto_ports = HashMap::new();
	for (key, port) in &metadata
		.network
		.as_ref()
		.expect("should have network")
		.ports
	{
		proto_ports.insert(
			key.clone(),
			Port {
				internal_port: port.internal_port,
				public_hostname: port.public_hostname.clone(),
				public_port: port.public_port,
				public_path: port.public_path.clone(),
				routing: Some(Routing {
					routing: Some(match &port.routing {
						pegboard::types::Routing::GameGuard { protocol } => {
							routing::Routing::GameGuard(routing::GameGuard {
								protocol: match protocol {
									pegboard::types::GameGuardProtocol::Http => {
										GameGuardProtocol::GgHttp as i32
									}
									pegboard::types::GameGuardProtocol::Https => {
										GameGuardProtocol::GgHttps as i32
									}
									pegboard::types::GameGuardProtocol::Tcp => {
										GameGuardProtocol::GgTcp as i32
									}
									pegboard::types::GameGuardProtocol::TcpTls => {
										GameGuardProtocol::GgTcpTls as i32
									}
									pegboard::types::GameGuardProtocol::Udp => {
										GameGuardProtocol::GgUdp as i32
									}
								},
							})
						}
						pegboard::types::Routing::Host { protocol } => {
							routing::Routing::Host(routing::Host {
								protocol: match protocol {
									pegboard::types::HostProtocol::Tcp => {
										HostProtocol::HostTcp as i32
									}
									pegboard::types::HostProtocol::Udp => {
										HostProtocol::HostUdp as i32
									}
								},
							})
						}
					}),
				}),
			},
		);
	}
	let network = actor_metadata::Network { ports: proto_ports };

	// Create the final ActorMetadata
	ActorMetadata {
		actor: Some(actor_metadata::Actor {
			actor_id: metadata.actor.actor_id.to_string(),
			tags: metadata.actor.tags.clone().into(),
			create_ts: metadata.actor.create_ts,
		}),
		network: Some(network),
		project: Some(actor_metadata::Project {
			project_id: metadata.project.project_id.to_string(),
			slug: metadata.project.slug.clone(),
		}),
		environment: Some(actor_metadata::Environment {
			env_id: metadata.environment.env_id.to_string(),
			slug: metadata.environment.slug.clone(),
		}),
		datacenter: Some(actor_metadata::Datacenter {
			name_id: metadata.datacenter.name_id.clone(),
			display_name: metadata.datacenter.display_name.clone(),
		}),
		cluster: Some(actor_metadata::Cluster {
			cluster_id: metadata.cluster.cluster_id.to_string(),
		}),
		build: Some(actor_metadata::Build {
			build_id: metadata.build.build_id.to_string(),
		}),
	}
}
