use std::convert::TryInto;

use chirp_workflow::prelude::*;
use fdb_util::{FormalKey, SERIALIZABLE};
use foundationdb as fdb;
use futures_util::FutureExt;
use nix::sys::signal::Signal;
use pegboard::{client_config, metrics, protocol, protocol::ClientFlavor, system_info};
use rivet_api::apis::{
	configuration::Configuration,
	core_intercom_pegboard_api::core_intercom_pegboard_mark_client_registered,
};
use sqlite_util::SqlitePoolExt;

use crate::keys;

mod migrations;

/// How long to wait after last ping before forcibly removing a client from the database and deleting its
/// workflow, evicting all actors. Note that the client may still be running and can reconnect.
const CLIENT_LOST_THRESHOLD_MS: i64 = util::duration::minutes(2);

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
	pub client_id: Uuid,
	pub flavor: ClientFlavor,
}

#[workflow]
pub async fn pegboard_client(ctx: &mut WorkflowCtx, input: &Input) -> GlobalResult<()> {
	migrations::run(ctx).await?;

	ctx.activity(InsertDbInput {
		flavor: input.flavor,
	})
	.await?;

	ctx.activity(PublishRegisteredInput {
		client_id: input.client_id,
	})
	.await?;

	ctx.loope(State::default(), |ctx, state| {
		let client_id = input.client_id;
		let flavor = input.flavor;

		async move {
			match ctx
				.listen_with_timeout::<Main>(CLIENT_LOST_THRESHOLD_MS)
				.await?
			{
				Some(Main::Forward(sig)) => {
					match sig {
						protocol::ToServer::Init {
							last_command_idx,
							config,
							system,
						} => {
							let allocable_memory = system.memory.total_memory / 1024 / 1024
								- config.reserved_resources.memory;

							let init_data = ctx
								.activity(ProcessInitInput {
									config,
									system,
									last_command_idx,
								})
								.await?;

							// Send init packet
							ctx.msg(ToWs {
								client_id,
								inner: protocol::ToClient::Init {
									last_event_idx: init_data.last_event_idx,
								},
							})
							.send()
							.await?;

							// Send missed commands
							if !init_data.missed_commands.is_empty() {
								ctx.msg(ToWs {
									client_id,
									inner: protocol::ToClient::Commands(init_data.missed_commands),
								})
								.send()
								.await?;
							}

							ctx.activity(InsertFdbInput {
								client_id,
								allocable_memory,
								flavor,
							})
							.await?;
						}
						// We assume events are in order by index
						protocol::ToServer::Events(events) => {
							// Write to db
							ctx.activity(InsertEventsInput {
								client_id,
								events: events.clone(),
							})
							.await?;

							// NOTE: This should not be parallelized because signals should be sent in order
							// Forward to actor workflows
							for event in events {
								#[allow(irrefutable_let_patterns)]
								if let protocol::Event::ActorStateUpdate { actor_id, state } =
									event.inner.deserialize()?
								{
									ctx.signal(crate::workflows::actor::StateUpdate {
										state,
										ignore_future_state: false,
									})
									.to_workflow::<crate::workflows::actor::Workflow>()
									.tag("actor_id", actor_id)
									.send()
									.await?;
								}
							}
						}
					}
				}
				Some(Main::Command(command)) => {
					handle_commands(ctx, client_id, state.drain_timeout_ts, vec![command]).await?;
				}
				Some(Main::PrewarmImage(sig)) => {
					ctx.msg(ToWs {
						client_id,
						inner: protocol::ToClient::PrewarmImage {
							image_id: sig.image_id,
							image_artifact_url_stub: sig.image_artifact_url_stub,
						},
					})
					.send()
					.await?;
				}
				Some(Main::Drain(sig)) => {
					state.drain_timeout_ts = Some(sig.drain_timeout_ts);

					ctx.activity(SetDrainInput {
						client_id,
						flavor,
						draining: true,
					})
					.await?;
				}
				Some(Main::Undrain(_)) => {
					state.drain_timeout_ts = None;

					ctx.activity(SetDrainInput {
						client_id,
						flavor,
						draining: false,
					})
					.await?;
				}
				None => {
					if ctx.activity(CheckExpiredInput {}).await? {
						return Ok(Loop::Break(()));
					}
				}
			}

			Ok(Loop::Continue)
		}
		.boxed()
	})
	.await?;

	ctx.activity(ClearFdbInput {
		client_id: input.client_id,
		flavor: input.flavor,
	})
	.await?;

	let actor_ids = ctx.activity(FetchAllActorsInput {}).await?;

	// Set all remaining actors as lost
	for actor_id in actor_ids {
		ctx.signal(crate::workflows::actor::StateUpdate {
			state: protocol::ActorState::Lost,
			ignore_future_state: false,
		})
		.to_workflow::<crate::workflows::actor::Workflow>()
		.tag("actor_id", actor_id)
		.send()
		.await?;
	}

	// Close websocket connection (its unlikely to be open)
	ctx.msg(CloseWs {
		client_id: input.client_id,
	})
	.send()
	.await?;

	Ok(())
}

#[derive(Serialize, Deserialize, Default)]
struct State {
	drain_timeout_ts: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct InsertDbInput {
	flavor: ClientFlavor,
}

#[activity(InsertDb)]
async fn insert_db(ctx: &ActivityCtx, input: &InsertDbInput) -> GlobalResult<()> {
	let pool = ctx.sqlite().await?;

	sql_execute!(
		[ctx, pool]
		"
		INSERT INTO state (create_ts, flavor)
		VALUES (?, ?)
		",
		util::timestamp::now(),
		input.flavor as i32,
	)
	.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct InsertFdbInput {
	client_id: Uuid,
	/// MiB.
	allocable_memory: u64,
	flavor: ClientFlavor,
}

#[activity(InsertFdb)]
async fn insert_fdb(ctx: &ActivityCtx, input: &InsertFdbInput) -> GlobalResult<()> {
	ctx.fdb()
		.await?
		.run(|tx, _mc| async move {
			let remaining_mem_key = keys::client::RemainingMemKey::new(input.client_id);
			let last_ping_key = keys::client::LastPingTsKey::new(input.client_id);

			// See if key already exists
			if tx
				.get(&keys::subspace().pack(&remaining_mem_key), SERIALIZABLE)
				.await?
				.is_none()
			{
				// Set remaining memory
				tx.set(
					&keys::subspace().pack(&remaining_mem_key),
					&remaining_mem_key
						.serialize(input.allocable_memory)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);

				// Set last ping
				let last_ping_ts = util::timestamp::now();
				tx.set(
					&keys::subspace().pack(&last_ping_key),
					&last_ping_key
						.serialize(last_ping_ts)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);

				// Insert into index
				let allocation_key = keys::datacenter::ClientsByRemainingMemKey::new(
					input.flavor,
					input.allocable_memory,
					last_ping_ts,
					input.client_id,
				);
				tx.set(
					&keys::subspace().pack(&allocation_key),
					&allocation_key
						.serialize(ctx.workflow_id())
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);
			}

			Ok(())
		})
		.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct PublishRegisteredInput {
	client_id: Uuid,
}

#[activity(PublishRegistered)]
async fn publish_registered(ctx: &ActivityCtx, input: &PublishRegisteredInput) -> GlobalResult<()> {
	let edge = ctx.config().server()?.rivet.edge()?;

	let config = Configuration {
		base_path: edge.intercom_endpoint.clone(),
		bearer_access_token: edge.server_token.as_ref().map(|x| x.read().clone()),
		..Default::default()
	};

	core_intercom_pegboard_mark_client_registered(&config, &input.client_id.to_string()).await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct ProcessInitInput {
	last_command_idx: i64,
	config: client_config::ClientConfig,
	system: system_info::SystemInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProcessInitOutput {
	last_event_idx: i64,
	missed_commands: Vec<protocol::CommandWrapper>,
}

#[activity(ProcessInit)]
async fn process_init(
	ctx: &ActivityCtx,
	input: &ProcessInitInput,
) -> GlobalResult<ProcessInitOutput> {
	let pool = &ctx.sqlite().await?;

	let ((last_event_idx,), commands) = tokio::try_join!(
		sql_fetch_one!(
			[ctx, (i64,), pool]
			"
			UPDATE state
			SET config = jsonb(?), system_info = jsonb(?)
			RETURNING last_event_idx
			",
			serde_json::to_value(&input.config)?,
			serde_json::to_value(&input.system)?,
		),
		sql_fetch_all!(
			[ctx, (i64, String), pool]
			"
			SELECT idx, json(payload)
			FROM commands
			WHERE idx > ?
			ORDER BY idx ASC
			",
			input.last_command_idx,
		),
	)?;

	Ok(ProcessInitOutput {
		last_event_idx,
		missed_commands: commands
			.into_iter()
			.map(|(index, payload)| {
				Ok(protocol::CommandWrapper {
					index,
					inner: protocol::Raw::from_string(payload)?,
				})
			})
			.collect::<GlobalResult<_>>()?,
	})
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct InsertEventsInput {
	client_id: Uuid,
	events: Vec<protocol::EventWrapper>,
}

#[activity(InsertEvents)]
async fn insert_events(ctx: &ActivityCtx, input: &InsertEventsInput) -> GlobalResult<()> {
	let last_event_idx = if let Some(last_event_wrapper) = input.events.last() {
		last_event_wrapper.index
	} else {
		return Ok(());
	};

	let mut tx = ctx.sqlite().await?.begin_immediate().await?;

	// TODO(RVT-4450): `last_event_idx < $2` and `ON CONFLICT DO NOTHING` is a workaround
	sql_execute!(
		[ctx, @tx &mut tx]
		"
		UPDATE state
		SET last_event_idx = $2
		WHERE last_event_idx < $2
		",
		last_event_idx,
	)
	.await?;

	// TODO: Parallelize
	for event in &input.events {
		let res = sql_execute!(
			[ctx, @tx &mut tx]
			"
			INSERT INTO events (idx, payload, ack_ts)
			VALUES (?, jsonb(?), ?)
			ON CONFLICT (idx) DO NOTHING
			",
			event.index,
			&event.inner,
			util::timestamp::now(),
		)
		.await?;

		if res.rows_affected() == 0 {
			metrics::CLIENT_DUPLICATE_EVENT
				.with_label_values(&[&input.client_id.to_string(), &event.index.to_string()])
				.inc();
		}
	}

	tx.commit().await?;

	Ok(())
}

pub async fn handle_commands(
	ctx: &mut WorkflowCtx,
	client_id: Uuid,
	drain_timeout_ts: Option<i64>,
	commands: Vec<protocol::Command>,
) -> GlobalResult<()> {
	if commands.is_empty() {
		return Ok(());
	}

	let raw_commands = commands
		.iter()
		.map(protocol::Raw::new)
		.collect::<Result<Vec<_>, _>>()?;

	// Write to db
	let index = ctx
		.activity(InsertCommandsInput {
			commands: raw_commands.clone(),
		})
		.await?;

	// TODO: Send as a single message
	for (i, raw_command) in raw_commands.into_iter().enumerate() {
		let wrapped_command = protocol::CommandWrapper {
			index: index + i as i64,
			inner: raw_command,
		};

		// Forward signal to ws as message
		ctx.msg(ToWs {
			client_id,
			inner: protocol::ToClient::Commands(vec![wrapped_command]),
		})
		.send()
		.await?;
	}

	// NOTE: Cannot parallelize because these must be sent in order
	// Update actor state based on commands
	for command in commands {
		match command {
			protocol::Command::StartActor { actor_id, config } => {
				let actor_workflow_id = ctx
					.workflow(crate::workflows::actor::Input {
						actor_id,
						parent_workflow_id: ctx.workflow_id(),
						config: *config,
					})
					.tag("actor_id", actor_id)
					.dispatch()
					.await?;

				// If this start actor command was received after the client started draining, immediately
				// inform the actor wf that it is draining
				if let Some(drain_timeout_ts) = drain_timeout_ts {
					ctx.signal(crate::workflows::actor::StateUpdate {
						state: protocol::ActorState::Draining { drain_timeout_ts },
						ignore_future_state: false,
					})
					.to_workflow_id(actor_workflow_id)
					.send()
					.await?;
				}
			}
			protocol::Command::SignalActor {
				actor_id,
				signal,
				ignore_future_state,
				..
			} => {
				if matches!(signal.try_into()?, Signal::SIGTERM | Signal::SIGKILL) {
					ctx.signal(crate::workflows::actor::StateUpdate {
						state: protocol::ActorState::Stopping,
						ignore_future_state,
					})
					.to_workflow::<crate::workflows::actor::Workflow>()
					.tag("actor_id", actor_id)
					.send()
					.await?;
				}
			}
		}
	}

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct InsertCommandsInput {
	commands: Vec<protocol::Raw<protocol::Command>>,
}

#[activity(InsertCommands)]
async fn insert_commands(ctx: &ActivityCtx, input: &InsertCommandsInput) -> GlobalResult<i64> {
	let mut tx = ctx.sqlite().await?.begin_immediate().await?;

	let (last_command_index,) = sql_fetch_one!(
		[ctx, (i64,), @tx &mut tx]
		"
		UPDATE state
			SET last_command_idx = last_command_idx + ?
		RETURNING last_command_idx - ?
		",
		input.commands.len() as i64,
	)
	.await?;

	// TODO: Parallelize
	for (index, command) in input.commands.iter().enumerate() {
		sql_execute!(
			[ctx, @tx &mut tx]
			"
			INSERT INTO commands (
				idx,
				payload,
				create_ts
			)
			VALUES (?, jsonb(?), ?)
			",
			last_command_index + index as i64 + 1,
			command,
			util::timestamp::now(),
		)
		.await?;
	}

	tx.commit().await?;

	Ok(last_command_index)
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct SetDrainInput {
	client_id: Uuid,
	flavor: ClientFlavor,
	draining: bool,
}

#[activity(SetDrain)]
async fn set_drain(ctx: &ActivityCtx, input: &SetDrainInput) -> GlobalResult<()> {
	ctx.op(crate::ops::client::update_allocation_idx::Input {
		client_id: input.client_id,
		client_workflow_id: ctx.workflow_id(),
		flavor: input.flavor,
		action: if input.draining {
			crate::ops::client::update_allocation_idx::Action::ClearIdx
		} else {
			crate::ops::client::update_allocation_idx::Action::AddIdx
		},
	})
	.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct ClearFdbInput {
	client_id: Uuid,
	flavor: ClientFlavor,
}

#[activity(ClearFdb)]
async fn clear_fdb(ctx: &ActivityCtx, input: &ClearFdbInput) -> GlobalResult<()> {
	// Does not clear the data keys like last ping ts
	ctx.op(crate::ops::client::update_allocation_idx::Input {
		client_id: input.client_id,
		client_workflow_id: ctx.workflow_id(),
		flavor: input.flavor,
		action: crate::ops::client::update_allocation_idx::Action::ClearIdx,
	})
	.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct FetchAllActorsInput {}

#[activity(FetchAllActors)]
async fn fetch_all_actors(
	ctx: &ActivityCtx,
	input: &FetchAllActorsInput,
) -> GlobalResult<Vec<Uuid>> {
	let pool = ctx.sqlite().await?;

	let actor_ids = sql_fetch_all!(
		[ctx, (Uuid,), pool]
		"
		SELECT actor_id
		FROM actors
		WHERE
			stopping_ts IS NULL AND
			stop_ts IS NULL AND
			exit_ts IS NULL
		",
	)
	.await?
	.into_iter()
	.map(|(id,)| id)
	.collect();

	Ok(actor_ids)
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct CheckExpiredInput {}

#[activity(CheckExpired)]
async fn check_expired(ctx: &ActivityCtx, input: &CheckExpiredInput) -> GlobalResult<bool> {
	let pool = ctx.sqlite().await?;

	let (is_expired,) = sql_fetch_one!(
		[ctx, (bool,), pool]
		"
		SELECT last_ping_ts < ?
		FROM state
		",
		util::timestamp::now() - CLIENT_LOST_THRESHOLD_MS,
	)
	.await?;

	Ok(is_expired)
}

#[signal("pegboard_client_registered")]
pub struct Registered {}

#[message("pegboard_client_to_ws")]
pub struct ToWs {
	pub client_id: Uuid,
	pub inner: protocol::ToClient,
}

#[signal("pegboard_prewarm_image")]
pub struct PrewarmImage {
	pub image_id: Uuid,
	pub image_artifact_url_stub: String,
}
#[message("pegboard_client_close_ws")]
pub struct CloseWs {
	pub client_id: Uuid,
}

#[signal("pegboard_client_drain")]
pub struct Drain {
	drain_timeout_ts: i64,
}

#[signal("pegboard_client_undrain")]
pub struct Undrain {}

join_signal!(Main {
	Command(protocol::Command),
	// Forwarded from the ws to this workflow
	Forward(protocol::ToServer),
	PrewarmImage,
	Drain,
	Undrain,
});
