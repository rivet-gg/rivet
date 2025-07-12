use std::{
	path::Path,
	result::Result::{Err, Ok},
	sync::Arc,
	time::Duration,
};

use anyhow::*;
use bytes::Bytes;
use futures_util::{
	stream::{FuturesUnordered, SplitSink, SplitStream},
	FutureExt, SinkExt, StreamExt,
};
use indoc::indoc;
use nix::{
	errno::Errno,
	sys::signal::{kill, Signal},
	unistd::Pid,
};
use pegboard::protocol;
use pegboard_config::runner_protocol;
use sqlx::Acquire;
use tokio::{
	fs,
	net::UnixStream,
	sync::{broadcast, Mutex, RwLock},
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use uuid::Uuid;

use crate::{ctx::Ctx, utils};

mod oci_config;
mod partial_oci_config;
mod seccomp;
pub(crate) mod setup;

/// How often to check that a PID is still running when observing actor state.
const PID_POLL_INTERVAL: Duration = Duration::from_millis(1000);
/// How long before killing a runner with a socket if it has not pinged.
const PING_TIMEOUT: Duration = Duration::from_secs(5);
/// How long to wait for the runner socket to become ready before timing out.
const SOCKET_READY_TIMEOUT: Duration = Duration::from_secs(5);
/// How long to wait when getting the PID before timing out.
const GET_PID_TIMEOUT: Duration = Duration::from_secs(256);
// IMPORTANT: This cannot be just `rivet-` because this is used as a prefix to filter cgroup names
// in cadvisor.
//
// If this was "rivet-", we'd have to report on non-actor cgroups with cadvisor.
//
// See also packages/core/services/cluster/src/workflows/server/install/install_scripts/files/cadvisor_metric_exporter.sh & packages/core/api/actor/src/route/metrics.rs
pub const RIVET_CONTAINER_PREFIX: &str = "pegboard-actor-";

#[derive(sqlx::FromRow)]
pub struct ProxiedPortRow {
	label: String,
	source: i64,
	target: Option<i64>,
	protocol: i64,
}

#[derive(Debug)]
enum ObservationState {
	Exited,
	Running,
	Dead,
}

pub struct Runner {
	runner_id: Uuid,
	comms: Comms,
	config: protocol::RunnerConfig,

	pid: RwLock<Option<Pid>>,

	/// Used instead of polling loops for faster updates.
	bump_channel: broadcast::Sender<()>,

	// TODO: replace with a single stream for each actor?
	actor_proxy_tx: broadcast::Sender<(rivet_util::Id, u32, runner_protocol::ToActor)>,
}

impl Runner {
	pub fn new(runner_id: Uuid, comms: Comms, config: protocol::RunnerConfig) -> Self {
		Runner {
			runner_id,
			comms,
			config,
			pid: RwLock::new(None),
			bump_channel: broadcast::channel(2).0,
			actor_proxy_tx: broadcast::channel(16).0,
		}
	}

	pub fn from_pid(
		runner_id: Uuid,
		comms: Comms,
		config: protocol::RunnerConfig,
		pid: Pid,
	) -> Self {
		Runner {
			runner_id,
			comms,
			config,
			pid: RwLock::new(Some(pid)),
			bump_channel: broadcast::channel(1).0,
			actor_proxy_tx: broadcast::channel(16).0,
		}
	}

	fn bump(&self) {
		let _ = self.bump_channel.send(());
	}

	pub async fn attach_socket(self: &Arc<Self>, ctx: &Arc<Ctx>, stream: UnixStream) -> Result<()> {
		match &self.comms {
			Comms::Basic => bail!("attempt to attach socket to basic runner"),
			Comms::Socket(tx) => {
				tracing::info!(runner_id=?self.runner_id, "attaching socket");

				let mut guard = tx.lock().await;

				if let Some(existing_tx) = &mut *guard {
					tracing::info!(runner_id=?self.runner_id, "runner received another socket, closing old one");

					// Close the old socket
					let buf = runner_protocol::encode_frame(&runner_protocol::ToRunner::Close {
						reason: Some("replacing with new socket".into()),
					})?;

					if let Err(err) = existing_tx.send(buf.into()).await {
						tracing::error!(runner_id=?self.runner_id, ?err, "failed to close old socket");
					};

					tracing::info!(runner_id=?self.runner_id, "socket replaced");
				} else {
					tracing::info!(runner_id=?self.runner_id, "socket attached");
				}

				// Wrap the stream in a framed transport
				let framed = Framed::new(stream, runner_protocol::codec());
				let (tx, rx) = framed.split();

				*guard = Some(tx);
				self.bump();

				// TODO: need to kill old handler thread
				// Spawn a new thread to handle incoming messages
				let self2 = self.clone();
				let ctx2 = ctx.clone();
				tokio::task::spawn(async move {
					if let Err(err) = self2.receive_frames(&ctx2, rx).await {
						tracing::error!(runner_id=?self2.runner_id, ?err, "socket error, killing runner");

						if let Err(err) = self2.signal(&ctx2, Signal::SIGKILL).await {
							tracing::error!(runner_id=?self2.runner_id, %err, "failed to kill runner");
						}
					}
				});
			}
		}

		Ok(())
	}

	async fn receive_frames(
		&self,
		_ctx: &Ctx,
		mut ws_rx: SplitStream<Framed<UnixStream, LengthDelimitedCodec>>,
	) -> Result<()> {
		loop {
			let Ok(frame) = tokio::time::timeout(PING_TIMEOUT, ws_rx.next()).await else {
				bail!("runner socket ping timed out");
			};

			let Some(buf) = frame else {
				tracing::debug!(runner_id=?self.runner_id, "runner socket closed");
				break Ok(());
			};

			let (_, packet) = runner_protocol::decode_frame::<runner_protocol::ToManager>(&buf?)
				.context("failed to decode frame")?;

			tracing::debug!(?packet, "runner received packet");

			match packet {
				runner_protocol::ToManager::Ping { .. } => {
					// TODO: Rate limit?
					self.send(&runner_protocol::ToRunner::Pong).await?;
				}
				runner_protocol::ToManager::ActorStateUpdate {
					actor_id,
					generation,
					state,
				} => {
					match self.config.image.allocation_type {
						protocol::ImageAllocationType::Single => {
							tracing::debug!("unexpected state update from non-multi runner");
						}
						protocol::ImageAllocationType::Multi => {
							// NOTE: We don't have to verify if the actor id given here is valid because only valid actors
							// are listening to this runner's `actor_proxy_tx`. This means invalid messages are ignored.
							// NOTE: No receivers is not an error
							let _ = self.actor_proxy_tx.send((
								actor_id,
								generation,
								runner_protocol::ToActor::StateUpdate { state },
							));
						}
					}
				}
				runner_protocol::ToManager::Kv(req) => {
					let _ = self.actor_proxy_tx.send((
						req.actor_id,
						req.generation,
						runner_protocol::ToActor::Kv(req),
					));
				}
			}
		}
	}

	pub async fn send(&self, packet: &runner_protocol::ToRunner) -> Result<()> {
		match &self.comms {
			Comms::Basic => bail!("cannot send socket message to basic runner"),
			Comms::Socket(socket) => {
				let mut sub = self.bump_channel.subscribe();

				// Wait for socket to connect in a retry loop
				let mut guard = tokio::time::timeout(SOCKET_READY_TIMEOUT, async {
					loop {
						{
							let guard = socket.lock().await;
							if guard.is_some() {
								break anyhow::Ok(guard);
							}
						}

						tracing::warn!(
							runner_id=?self.runner_id,
							"socket not yet attached, can't send message. retrying",
						);

						sub.recv().await.context("bump channel closed")?;
					}
				})
				.await
				.with_context(|| {
					format!(
						"timed out waiting for runner {} socket to attach",
						self.runner_id
					)
				})??;

				let socket = guard.as_mut().expect("should exist");
				let buf =
					runner_protocol::encode_frame(packet).context("failed to encode frame")?;
				socket
					.send(buf.into())
					.await
					.context("failed to send packet to socket")?;
			}
		}

		Ok(())
	}

	pub async fn start(
		self: &Arc<Self>,
		ctx: &Arc<Ctx>,
	) -> Result<protocol::HashableMap<String, protocol::ProxiedPort>> {
		tracing::info!(runner_id=?self.runner_id, "starting");

		// Write runner to DB
		let config_json = serde_json::to_vec(&self.config)?;

		utils::sql::query(|| async {
			// NOTE: On conflict here in case this query runs but the command is not acknowledged
			sqlx::query(indoc!(
				"
				INSERT INTO runners (
					runner_id,
					comms,
					config,
					image_id,
					start_ts
				)
				VALUES (?1, ?2, ?3, ?4, ?5)
				ON CONFLICT (runner_id) DO NOTHING
				",
			))
			.bind(self.runner_id)
			.bind(if self.has_socket() {
				setup::Comms::Socket
			} else {
				setup::Comms::Basic
			} as i32)
			.bind(&config_json)
			.bind(self.config.image.id)
			.bind(utils::now())
			.execute(&mut *ctx.sql().await?)
			.await
		})
		.await?;

		// Setup needs to occur outside of spawned task because the ports are returned
		let proxied_ports = match self.setup(&ctx).await {
			Ok(proxied_ports) => proxied_ports,
			Err(err) => {
				tracing::error!(runner_id=?self.runner_id, ?err, "setup failed");

				// Cleanup afterwards
				if let Err(err) = self.cleanup(&ctx).await {
					tracing::error!(runner_id=?self.runner_id, ?err, "cleanup failed");
				}

				return Err(err);
			}
		};

		// Lifecycle
		let self2 = self.clone();
		let ctx2 = ctx.clone();
		tokio::spawn(async move {
			match self2.run(&ctx2).await {
				Ok(_) => {
					if let Err(err) = self2.observe(&ctx2, false).await {
						tracing::error!(runner_id=?self2.runner_id, ?err, "observe failed");
					}
				}
				Err(err) => {
					tracing::error!(runner_id=?self2.runner_id, ?err, "run failed")
				}
			}

			// Cleanup afterwards
			if let Err(err) = self2.cleanup(&ctx2).await {
				tracing::error!(runner_id=?self2.runner_id, ?err, "cleanup failed");
			}
		});

		Ok(proxied_ports)
	}

	async fn run(&self, ctx: &Ctx) -> Result<()> {
		// NOTE: This is the env that goes to the container-runner process, NOT the env that is inserted into
		// the container.
		let mut runner_env = vec![
			(
				"ROOT_USER_ENABLED",
				if self.config.root_user_enabled {
					"1"
				} else {
					"0"
				}
				.to_string(),
			),
			("RUNNER_ID", self.runner_id.to_string()),
			(
				"ENVIRONMENT_ID",
				self.metadata.environment.env_id.to_string(),
			),
		];

		if let Some(vector) = &ctx.config().vector {
			runner_env.push(("VECTOR_SOCKET_ADDR", vector.address.to_string()));
		}

		self.spawn_orphaned(ctx, &runner_env).await
	}

	// Silent prevents dupe logs, this function is called for every actor running on this runner as well as
	// for the runner's observer task
	pub async fn observe(&self, ctx: &Ctx, silent: bool) -> Result<Option<i32>> {
		let pid = self.pid().await?;

		let runner_path = ctx.runner_path(self.runner_id);
		let exit_code_path = runner_path.join("exit-code");
		let proc_path = Path::new("/proc").join(pid.to_string());

		let mut futs = FuturesUnordered::new();

		// Watch for exit code file being written
		futs.push(
			async {
				utils::wait_for_write(&exit_code_path).await?;

				anyhow::Ok(ObservationState::Exited)
			}
			.boxed(),
		);

		// Polling interval to check that the pid still exists
		futs.push(
			async {
				tokio::time::sleep(PID_POLL_INTERVAL).await;

				if fs::metadata(&proc_path).await.is_ok() {
					anyhow::Ok(ObservationState::Running)
				} else {
					anyhow::Ok(ObservationState::Dead)
				}
			}
			.boxed(),
		);

		let state = loop {
			// Get next complete future
			let state = futs
				.next()
				.await
				.context("observation failed, developer error")??;

			// If still running, add poll future back to list
			if let ObservationState::Running = state {
				futs.push(
					async {
						tokio::time::sleep(PID_POLL_INTERVAL).await;

						if fs::metadata(&proc_path).await.is_ok() {
							Ok(ObservationState::Running)
						} else {
							Ok(ObservationState::Dead)
						}
					}
					.boxed(),
				);
			} else {
				break state;
			}
		};

		let exit_code = if let ObservationState::Exited = state {
			use std::result::Result::Ok;
			match fs::read_to_string(&exit_code_path).await {
				Ok(contents) => match contents.trim().parse::<i32>() {
					Ok(x) => Some(x),
					Err(err) => {
						if !silent {
							tracing::error!(runner_id=?self.runner_id, ?err, "failed to parse exit code file");
						}

						None
					}
				},
				Err(err) => {
					if !silent {
						tracing::error!(runner_id=?self.runner_id, ?err, "failed to read exit code file");
					}

					None
				}
			}
		} else {
			if !silent {
				tracing::warn!(runner_id=?self.runner_id, "process died before exit code file was written");
			}

			None
		};

		if !silent {
			tracing::info!(runner_id=?self.runner_id, ?exit_code, "exited");
		}

		self.set_exit_code(ctx, exit_code).await?;

		Ok(exit_code)
	}

	pub fn new_actor_proxy(&self, actor_id: rivet_util::Id, generation: u32) -> ActorProxy {
		ActorProxy::new(actor_id, generation, self.actor_proxy_tx.subscribe())
	}

	pub async fn signal(&self, ctx: &Ctx, signal: Signal) -> Result<()> {
		// https://pubs.opengroup.org/onlinepubs/9699919799/functions/kill.html
		if (signal as i32) < 1 {
			bail!("signals < 1 not allowed");
		}

		let pid = self.pid().await?;

		match kill(pid, signal) {
			Ok(_) => {}
			Err(Errno::ESRCH) => {
				tracing::warn!(?pid, "pid not found for signalling")
			}
			Err(err) => return Err(err.into()),
		}

		// Update DB
		utils::sql::query(|| async {
			let mut conn = ctx.sql().await?;
			let mut tx = conn.begin().await?;

			sqlx::query(indoc!(
				"
				UPDATE runners
				SET
					stop_ts = ?2
				WHERE
					runner_id = ?1 AND
					stop_ts IS NULL
				",
			))
			.bind(self.runner_id)
			.bind(utils::now())
			.execute(&mut *tx)
			.await?;

			// Update LRU cache
			sqlx::query(indoc!(
				"
				UPDATE images_cache
				SET last_used_ts = ?2
				WHERE image_id = ?1
				",
			))
			.bind(self.config.image.id)
			.bind(utils::now())
			.execute(&mut *tx)
			.await?;

			tx.commit().await?;

			Ok(())
		})
		.await?;

		Ok(())
	}

	#[tracing::instrument(skip_all)]
	pub async fn set_exit_code(&self, ctx: &Ctx, exit_code: Option<i32>) -> Result<()> {
		// Update DB
		utils::sql::query(|| async {
			sqlx::query(indoc!(
				"
				UPDATE runners
				SET
					exit_ts = ?2,
					exit_code = ?3
				WHERE
					runner_id = ?1 AND
					exit_ts IS NULL
				",
			))
			.bind(self.runner_id)
			.bind(utils::now())
			.bind(exit_code)
			.execute(&mut *ctx.sql().await?)
			.await
		})
		.await?;

		Ok(())
	}

	#[tracing::instrument(skip_all)]
	pub async fn cleanup(&self, ctx: &Ctx) -> Result<()> {
		tracing::info!(runner_id=?self.runner_id, "cleaning up runner");

		// Set exit code if it hasn't already been set
		self.set_exit_code(ctx, None).await?;

		// Unbind ports
		utils::sql::query(|| async {
			sqlx::query(indoc!(
				"
				UPDATE runner_ports
				SET delete_ts = ?3
				WHERE
					runner_id = ?1
				",
			))
			.bind(self.runner_id)
			.bind(utils::now())
			.execute(&mut *ctx.sql().await?)
			.await
		})
		.await?;

		// Cleanup setup. Should only be called after the exit code is set successfully for consistent state
		self.cleanup_setup(ctx).await;

		// It is important that we remove from the runners list last so that we prevent duplicates
		{
			let mut runners = ctx.runners.write().await;
			runners.remove(&self.runner_id);
		}

		Ok(())
	}
}

impl Runner {
	pub fn config(&self) -> &protocol::RunnerConfig {
		&self.config
	}

	pub fn container_id(&self) -> String {
		format!("{RIVET_CONTAINER_PREFIX}{}", self.runner_id)
	}

	pub async fn ports(
		&self,
		ctx: &Ctx,
	) -> Result<protocol::HashableMap<String, protocol::ProxiedPort>> {
		let rows = utils::sql::query(|| async {
			sqlx::query_as::<_, ProxiedPortRow>(indoc!(
				"
				SELECT label, source, target, protocol FROM runner_ports
				WHERE
					runner_id = ?1 AND
					delete_ts IS NULL
				",
			))
			.bind(self.runner_id)
			.fetch_all(&mut *ctx.sql().await?)
			.await
		})
		.await?;

		rows.into_iter()
			.map(|row| {
				let source = row.source.try_into()?;

				Ok((
					row.label,
					protocol::ProxiedPort {
						source,
						target: row
							.target
							.map(TryInto::try_into)
							.transpose()?
							.unwrap_or(source),
						lan_hostname: ctx.config().network.lan_hostname.clone(),
						protocol: protocol::TransportProtocol::from_repr(row.protocol.try_into()?)
							.context("bad port protocol")?,
					},
				))
			})
			.collect()
	}

	pub async fn pid(&self) -> Result<Pid> {
		let mut sub = self.bump_channel.subscribe();
		let mut i = 0;

		tokio::time::timeout(GET_PID_TIMEOUT, async {
			loop {
				{
					if let Some(pid) = *self.pid.read().await {
						break anyhow::Ok(pid);
					}
				}

				// Progress log
				if i % 10 == 0 {
					tracing::warn!(
						runner_id=?self.runner_id,
						"waiting for pid of runner",
					);
				}

				i += 1;

				sub.recv().await.context("bump channel closed")?;
			}
		})
		.await
		.with_context(|| {
			format!(
				"timed out waiting for runner {} to get PID, considering runner stopped",
				self.runner_id,
			)
		})?
	}

	pub fn has_socket(&self) -> bool {
		matches!(self.comms, Comms::Socket(_))
	}
}

pub enum Comms {
	Basic,
	Socket(Mutex<Option<SplitSink<Framed<UnixStream, LengthDelimitedCodec>, Bytes>>>),
}

impl Comms {
	pub fn socket() -> Self {
		Comms::Socket(Mutex::new(None))
	}
}

pub struct ActorProxy {
	actor_id: rivet_util::Id,
	generation: u32,
	sub: broadcast::Receiver<(rivet_util::Id, u32, runner_protocol::ToActor)>,
}

impl ActorProxy {
	fn new(
		actor_id: rivet_util::Id,
		generation: u32,
		sub: broadcast::Receiver<(rivet_util::Id, u32, runner_protocol::ToActor)>,
	) -> Self {
		ActorProxy {
			actor_id,
			generation,
			sub,
		}
	}
	pub async fn next(&mut self) -> Option<runner_protocol::ToActor> {
		loop {
			let Ok((other_actor_id, other_generation, state)) = self.sub.recv().await else {
				tracing::error!("actor observer channel dropped");

				return None;
			};

			if self.actor_id == other_actor_id && self.generation == other_generation {
				return Some(state);
			}
		}
	}
}
