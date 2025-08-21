use std::{
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
							.send(&runner_protocol::ToRunner::StartActor {
								actor_id: self.actor_id,
								generation: self.generation,
								env: self.config.env.clone(),
								metadata: self.config.metadata.clone(),
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
							.send(&runner_protocol::ToRunner::StartActor {
								actor_id: self.actor_id,
								generation: self.generation,
								env: self.config.env.clone(),
								metadata: self.config.metadata.clone(),
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
							match state {
								runner_protocol::ActorState::Running => {
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
								runner_protocol::ActorState::Exited {
									exit_code,
								} => break exit_code,
							}
						}
						runner_protocol::ToActor::Kv(req) => {
							// TODO: Add queue and bg thread for processing kv ops
							// Run kv operation
							match req.data {
								runner_protocol::KvRequestData::Get { keys } => {
									let res = self.kv.get(keys).await;
									let error = res.as_ref().err().map(|x| x.to_string());

									self.runner.send(&runner_protocol::ToRunner::Kv(runner_protocol::KvResponse {
										request_id: req.request_id,
										data: res.ok().map(|entries| {
											let (keys, values) = entries.into_iter().unzip();
											runner_protocol::KvResponseData::Get {
												keys,
												values,
											}
										}),
										error,
									})).await?;
								}
								runner_protocol::KvRequestData::List { query, reverse, limit } => {
									let res = self.kv.list(query, reverse, limit).await;
									let error = res.as_ref().err().map(|x| x.to_string());

									self.runner.send(&runner_protocol::ToRunner::Kv(runner_protocol::KvResponse {
										request_id: req.request_id,
										data: res.ok().map(|entries| {
											let (keys, values) = entries.into_iter().unzip();
											runner_protocol::KvResponseData::List {
												keys,
												values,
											}
										}),
										error,
									})).await?;
								}
								runner_protocol::KvRequestData::Put { keys, values } => {
									let res = self.kv.put(keys.into_iter().zip(values.into_iter()).collect()).await;
									let error = res.as_ref().err().map(|x| x.to_string());

									self.runner.send(&runner_protocol::ToRunner::Kv(runner_protocol::KvResponse {
										request_id: req.request_id,
										data: res.ok().map(|_| runner_protocol::KvResponseData::Put {}),
										error,
									})).await?;
								}
								runner_protocol::KvRequestData::Delete { keys } => {
									let res = self.kv.delete(keys).await;
									let error = res.as_ref().err().map(|x| x.to_string());

									self.runner.send(&runner_protocol::ToRunner::Kv(runner_protocol::KvResponse {
										request_id: req.request_id,
										data: res.ok().map(|_| runner_protocol::KvResponseData::Delete {}),
										error,
									})).await?;
								}
								runner_protocol::KvRequestData::Drop { } => {
									let res = self.kv.delete_all().await;
									let error = res.as_ref().err().map(|x| x.to_string());

									self.runner.send(&runner_protocol::ToRunner::Kv(runner_protocol::KvResponse {
										request_id: req.request_id,
										data: res.ok().map(|_| runner_protocol::KvResponseData::Drop {}),
										error,
									})).await?;
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
					.send(&runner_protocol::ToRunner::SignalActor {
						actor_id: self.actor_id,
						generation: self.generation,
						signal: signal as i32,
						persist_storage,
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
