use std::convert::TryInto;

use ::build::types::BuildRuntime;
use chirp_workflow::prelude::*;
use fdb_util::{end_of_key_range, FormalKey, SERIALIZABLE, SNAPSHOT};
use foundationdb::{
	self as fdb,
	options::{ConflictRangeType, StreamingMode},
};
use futures_util::{FutureExt, StreamExt, TryStreamExt};
use nix::sys::signal::Signal;
use rivet_api::{
	apis::{
		configuration::Configuration,
		core_intercom_pegboard_api::core_intercom_pegboard_mark_client_registered,
	},
	models,
};
use rivet_operation::prelude::proto::{self, backend::pkg::*};
use sqlx::Acquire;

use crate::{
	client_config, keys, metrics, protocol, protocol::ClientFlavor, system_info,
	workflows::actor::Allocate,
};

mod migrations;

/// How long after last ping before not considering a client for allocation.
pub const CLIENT_ELIGIBLE_THRESHOLD_MS: i64 = util::duration::seconds(10);
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
							last_workflow_id,
							config,
							system,
						} => {
							let init_data = ctx
								.activity(ProcessInitInput {
									config: config.clone(),
									system: system.clone(),
									// Ignore init packet if workflow id doesn't match. Manager will reset
									last_command_idx: if last_workflow_id
										.map(|id| id != ctx.workflow_id())
										.unwrap_or_default()
									{
										-1
									} else {
										last_command_idx
									},
								})
								.await?;

							// Send init packet
							ctx.msg(ToWs {
								client_id,
								inner: protocol::ToClient::Init {
									last_event_idx: init_data.last_event_idx,
									workflow_id: ctx.workflow_id(),
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
								flavor,
								config,
								system,
							})
							.await?;

							let (res, _) = ctx
								.join((
									// Check for pending actors as soon as connected
									v(2).activity(AllocatePendingActorsInput {}),
									activity(UpdateMetricsInput {
										client_id,
										flavor,
										draining: state.drain_timeout_ts.is_some(),
										clear: false,
									}),
								))
								.await?;

							// Dispatch pending allocs
							for alloc in res.allocations {
								ctx.v(2)
									.signal(alloc.signal)
									.to_workflow::<crate::workflows::actor::Workflow>()
									.tag("actor_id", alloc.actor_id)
									.send()
									.await?;
							}
						}
						// Events are in order by index
						protocol::ToServer::Events(events) => {
							// Write to db
							ctx.join((
								activity(InsertEventsInput {
									client_id,
									events: events.clone(),
								}),
								activity(UpdateMetricsInput {
									client_id,
									flavor,
									draining: state.drain_timeout_ts.is_some(),
									clear: false,
								}),
							))
							.await?;

							// NOTE: This should not be parallelized because signals should be sent in order
							// Forward to actor workflows
							for event in events {
								#[allow(irrefutable_let_patterns)]
								if let protocol::Event::ActorStateUpdate {
									actor_id,
									generation,
									state,
								} = event.inner.deserialize()?
								{
									// Try actor2 first
									let res = ctx
										.signal(crate::workflows::actor::StateUpdate {
											generation,
											state: state.clone(),
										})
										.to_workflow::<crate::workflows::actor::Workflow>()
										.tag("actor_id", actor_id)
										.send()
										.await;

									if let Some(WorkflowError::WorkflowNotFound) =
										res.as_workflow_error()
									{
										// Try old actors
										let res = ctx
											.signal(crate::workflows::actor::v1::StateUpdate {
												generation,
												state,
											})
											.to_workflow::<crate::workflows::actor::v1::Workflow>()
											.tag("actor_id", actor_id)
											.send()
											.await;

										if let Some(WorkflowError::WorkflowNotFound) =
											res.as_workflow_error()
										{
											tracing::warn!(
												?actor_id,
												"actor workflow not found, likely already stopped"
											);
										} else {
											res?;
										}
									} else {
										res?;
									}
								}
							}
						}
						protocol::ToServer::AckCommands { last_command_idx } => {
							ctx.activity(AckCommandsInput { last_command_idx }).await?;
						}
					}
				}
				Some(Main::Command(command)) => {
					handle_commands(
						ctx,
						client_id,
						flavor,
						state.drain_timeout_ts,
						vec![command],
					)
					.await?;
				}
				Some(Main::PrewarmImage2(sig)) => {
					ctx.msg(ToWs {
						client_id,
						inner: protocol::ToClient::PrewarmImage { image: sig.image },
					})
					.send()
					.await?;
				}
				Some(Main::Drain(sig)) => {
					state.drain_timeout_ts = Some(sig.drain_timeout_ts);

					ctx.join((
						activity(SetDrainInput {
							client_id,
							flavor,
							draining: true,
						}),
						v(2).activity(UpdateMetricsInput {
							client_id,
							flavor,
							draining: true,
							clear: false,
						}),
					))
					.await?;
				}
				Some(Main::Undrain(_)) => {
					state.drain_timeout_ts = None;

					ctx.join((
						activity(SetDrainInput {
							client_id,
							flavor,
							draining: false,
						}),
						v(2).activity(UpdateMetricsInput {
							client_id,
							flavor,
							draining: false,
							clear: false,
						}),
					))
					.await?;

					let actor_ids = ctx
						.activity(FetchRemainingActorsInput { client_id })
						.await?;

					// Undrain all remaining actors
					for actor_id in actor_ids {
						// Try actor2 first
						let res = ctx
							.signal(crate::workflows::actor::Undrain {})
							.to_workflow::<crate::workflows::actor::Workflow>()
							.tag("actor_id", actor_id)
							.send()
							.await;

						if let Some(WorkflowError::WorkflowNotFound) = res.as_workflow_error() {
							// Try old actors
							let res = ctx
								.signal(crate::workflows::actor::v1::Undrain {})
								.to_workflow::<crate::workflows::actor::v1::Workflow>()
								.tag("actor_id", actor_id)
								.send()
								.await;

							if let Some(WorkflowError::WorkflowNotFound) = res.as_workflow_error() {
								tracing::debug!(
									?actor_id,
									"actor workflow not found to undrain, likely already stopped"
								);
							} else {
								res?;
							}
						} else {
							res?;
						}
					}

					// Check for pending actors
					let res = ctx.v(2).activity(AllocatePendingActorsInput {}).await?;

					// Dispatch pending allocs
					for alloc in res.allocations {
						ctx.v(2)
							.signal(alloc.signal)
							.to_workflow::<crate::workflows::actor::Workflow>()
							.tag("actor_id", alloc.actor_id)
							.send()
							.await?;
					}
				}
				Some(Main::CheckQueue(_)) => {
					// Check for pending actors
					let res = ctx.v(2).activity(AllocatePendingActorsInput {}).await?;

					// Dispatch pending allocs
					for alloc in res.allocations {
						ctx.v(2)
							.signal(alloc.signal)
							.to_workflow::<crate::workflows::actor::Workflow>()
							.tag("actor_id", alloc.actor_id)
							.send()
							.await?;
					}
				}
				None => {
					if ctx.activity(CheckExpiredInput { client_id }).await? {
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

	ctx.activity(UpdateMetricsInput {
		client_id: input.client_id,
		flavor: input.flavor,
		draining: false,
		clear: true,
	})
	.await?;

	let actors = ctx
		.activity(FetchRemainingActorsInput {
			client_id: input.client_id,
		})
		.await?;

	// Set all remaining actors as lost
	for (actor_id, generation) in actors {
		// Try actor2 first
		let res = ctx
			.signal(crate::workflows::actor::StateUpdate {
				generation,
				state: protocol::ActorState::Lost,
			})
			.to_workflow::<crate::workflows::actor::Workflow>()
			.tag("actor_id", actor_id)
			.send()
			.await;

		if let Some(WorkflowError::WorkflowNotFound) = res.as_workflow_error() {
			// Try old actors
			let res = ctx
				.signal(crate::workflows::actor::v1::StateUpdate {
					generation,
					state: protocol::ActorState::Lost,
				})
				.to_workflow::<crate::workflows::actor::v1::Workflow>()
				.tag("actor_id", actor_id)
				.send()
				.await;

			if let Some(WorkflowError::WorkflowNotFound) = res.as_workflow_error() {
				tracing::warn!(
					?actor_id,
					"actor workflow not found, likely already stopped"
				);
			} else {
				res?;
			}
		} else {
			res?;
		}
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
	flavor: ClientFlavor,
	config: client_config::ClientConfig,
	system: system_info::SystemInfo,
}

#[activity(InsertFdb)]
async fn insert_fdb(ctx: &ActivityCtx, input: &InsertFdbInput) -> GlobalResult<()> {
	// MiB
	let allocable_memory =
		input.system.memory.total_memory / 1024 / 1024 - input.config.reserved_resources.memory;
	// Millicores
	let allocable_cpu = input.system.cpu.physical_core_count * 1000;

	ctx.fdb()
		.await?
		.run(|tx, _mc| async move {
			let remaining_mem_key = keys::client::RemainingMemoryKey::new(input.client_id);
			let remaining_cpu_key = keys::client::RemainingCpuKey::new(input.client_id);
			let last_ping_ts_key = keys::client::LastPingTsKey::new(input.client_id);
			let workflow_id_key = keys::client::WorkflowIdKey::new(input.client_id);

			let (remaining_mem_entry, last_ping_ts_entry, workflow_id_entry) = tokio::try_join!(
				tx.get(&keys::subspace().pack(&remaining_mem_key), SERIALIZABLE),
				tx.get(&keys::subspace().pack(&last_ping_ts_key), SERIALIZABLE),
				tx.get(&keys::subspace().pack(&workflow_id_key), SERIALIZABLE),
			)?;

			// See if key already exists
			let existing = if let (
				Some(remaining_mem_entry),
				Some(last_ping_ts_entry),
				Some(workflow_id_entry),
			) = (remaining_mem_entry, last_ping_ts_entry, workflow_id_entry)
			{
				let remaining_mem = remaining_mem_key
					.deserialize(&remaining_mem_entry)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
				let last_ping_ts = last_ping_ts_key
					.deserialize(&last_ping_ts_entry)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
				let workflow_id = workflow_id_key
					.deserialize(&workflow_id_entry)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

				if workflow_id == ctx.workflow_id() {
					Some((remaining_mem, last_ping_ts))
				} else {
					// Workflow id changed, reset state
					None
				}
			} else {
				// Initial insert
				None
			};

			let (remaining_mem, last_ping_ts) = if let Some(existing) = existing {
				existing
			} else {
				// Set workflow id
				tx.set(
					&keys::subspace().pack(&workflow_id_key),
					&workflow_id_key
						.serialize(ctx.workflow_id())
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);

				// Set remaining memory
				tx.set(
					&keys::subspace().pack(&remaining_mem_key),
					&remaining_mem_key
						.serialize(allocable_memory)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);

				// Set total memory
				let total_mem_key = keys::client::TotalMemoryKey::new(input.client_id);
				tx.set(
					&keys::subspace().pack(&total_mem_key),
					&total_mem_key
						.serialize(allocable_memory)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);

				// Set remaining cpu
				tx.set(
					&keys::subspace().pack(&remaining_cpu_key),
					// Millicores
					&remaining_cpu_key
						.serialize(allocable_cpu)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);

				// Set total cpu
				let total_cpu_key = keys::client::TotalCpuKey::new(input.client_id);
				tx.set(
					&keys::subspace().pack(&total_cpu_key),
					&total_cpu_key
						.serialize(allocable_cpu)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);

				// Set last ping
				let last_ping_ts = util::timestamp::now();
				tx.set(
					&keys::subspace().pack(&last_ping_ts_key),
					&last_ping_ts_key
						.serialize(last_ping_ts)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);

				(allocable_memory, last_ping_ts)
			};

			// Insert into index (same as the `update_allocation_idx` op with `AddIdx`)
			let allocation_key = keys::datacenter::ClientsByRemainingMemKey::new(
				input.flavor,
				remaining_mem,
				last_ping_ts,
				input.client_id,
			);
			tx.set(
				&keys::subspace().pack(&allocation_key),
				&allocation_key
					.serialize(ctx.workflow_id())
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			);

			Ok(())
		})
		.custom_instrument(tracing::info_span!("client_insert_tx"))
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

	// Create ephemeral token to authenticate with core
	let token_res = op!([ctx] token_create {
		token_config: Some(token::create::request::TokenConfig {
			ttl: util::duration::minutes(5),
		}),
		refresh_token_config: None,
		issuer: "pegboard_client".to_owned(),
		client: None,
		kind: Some(token::create::request::Kind::New(
			token::create::request::KindNew { entitlements: vec![proto::claims::Entitlement {
				kind: Some(proto::claims::entitlement::Kind::Bypass(
					proto::claims::entitlement::Bypass { }
				)),
			}]},
		)),
		label: Some("byp".to_owned()),
		ephemeral: true,
	})
	.await?;
	let token = unwrap!(token_res.token).token;

	let config = Configuration {
		client: rivet_pools::reqwest::client().await?,
		base_path: util::url::to_string_without_slash(&edge.intercom_address),
		bearer_access_token: Some(token),
		..Default::default()
	};

	core_intercom_pegboard_mark_client_registered(
		&config,
		&input.client_id.to_string(),
		models::CoreIntercomPegboardMarkClientRegisteredRequest {
			// TODO: Get server id from init packet. For now because the pb client id is the same as the
			// server id it doesn't matter
			server_id: input.client_id,
		},
	)
	.await?;

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

	let ((last_event_idx,), commands, _) = tokio::try_join!(
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
		// TODO: Added while sqlite flushing system is in place. As the database grows, flushes get slower
		// and slower.
		sql_execute!(
			[ctx, pool]
			"
			DELETE
			FROM commands
			WHERE idx <= ?
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

// TODO: Added while sqlite flushing system is in place. As the database grows, flushes get slower
// and slower.
#[derive(Debug, Serialize, Deserialize, Hash)]
struct AckCommandsInput {
	last_command_idx: i64,
}

#[activity(AckCommands)]
async fn ack_commands(ctx: &ActivityCtx, input: &AckCommandsInput) -> GlobalResult<()> {
	let pool = &ctx.sqlite().await?;

	sql_execute!(
		[ctx, pool]
		"
		DELETE
		FROM commands
		WHERE idx <= ?
		",
		input.last_command_idx,
	)
	.await?;

	Ok(())
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

	let pool = ctx.sqlite().await?;
	let mut conn = pool.conn().await?;
	let mut tx = conn.begin().await?;

	// TODO(RVT-4450): `last_event_idx < $2` and `ON CONFLICT DO NOTHING` is a workaround
	sql_execute!(
		[ctx, @tx &mut tx]
		"
		UPDATE state
		SET last_event_idx = ?
		WHERE last_event_idx < ?
		",
		last_event_idx,
		last_event_idx,
	)
	.await?;

	// TODO: Disabled while sqlite flushing system is in place. As the database grows, flushes get slower and
	// slower.
	// // TODO: Parallelize
	// for event in &input.events {
	// 	let res = sql_execute!(
	// 		[ctx, @tx &mut tx]
	// 		"
	// 		INSERT INTO events (idx, payload, ack_ts)
	// 		VALUES (?, jsonb(?), ?)
	// 		ON CONFLICT (idx) DO NOTHING
	// 		",
	// 		event.index,
	// 		&event.inner,
	// 		util::timestamp::now(),
	// 	)
	// 	.await?;

	// 	if res.rows_affected() == 0 {
	// 		metrics::CLIENT_DUPLICATE_EVENT
	// 			.with_label_values(&[&input.client_id.to_string(), &event.index.to_string()])
	// 			.inc();
	// 	}
	// }

	tx.commit().await?;

	Ok(())
}

pub async fn handle_commands(
	ctx: &mut WorkflowCtx,
	client_id: Uuid,
	flavor: ClientFlavor,
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
	let (index, _) = ctx
		.join((
			activity(InsertCommandsInput {
				commands: raw_commands.clone(),
			}),
			activity(UpdateMetricsInput {
				client_id,
				flavor,
				draining: drain_timeout_ts.is_some(),
				clear: false,
			}),
		))
		.await?;

	// Forward commands as a single message
	let wrapped_commands = raw_commands
		.into_iter()
		.enumerate()
		.map(|(i, raw_command)| protocol::CommandWrapper {
			index: index + i as i64,
			inner: raw_command,
		})
		.collect::<Vec<_>>();

	ctx.msg(ToWs {
		client_id,
		inner: protocol::ToClient::Commands(wrapped_commands),
	})
	.send()
	.await?;

	// NOTE: Cannot parallelize because these must be sent in order
	// Forward actor state based on commands
	for command in commands {
		match command {
			protocol::Command::StartActor { actor_id, .. } => {
				// If this start actor command was received after the client started draining, immediately
				// inform the actor wf that it is draining
				if let Some(drain_timeout_ts) = drain_timeout_ts {
					// Try actor2 first
					let res = ctx
						.signal(crate::workflows::actor::Drain { drain_timeout_ts })
						.to_workflow::<crate::workflows::actor::Workflow>()
						.tag("actor_id", actor_id)
						.send()
						.await;

					if let Some(WorkflowError::WorkflowNotFound) = res.as_workflow_error() {
						// Try old actors
						ctx.signal(crate::workflows::actor::v1::Drain { drain_timeout_ts })
							.to_workflow::<crate::workflows::actor::v1::Workflow>()
							.tag("actor_id", actor_id)
							.send()
							.await?;
					} else {
						res?;
					}
				}
			}
			protocol::Command::SignalActor {
				actor_id,
				generation,
				signal,
				..
			} => {
				if matches!(signal.try_into()?, Signal::SIGTERM | Signal::SIGKILL) {
					// Try actor2 first
					let res = ctx
						.signal(crate::workflows::actor::StateUpdate {
							generation,
							state: protocol::ActorState::Stopping,
						})
						.to_workflow::<crate::workflows::actor::Workflow>()
						.tag("actor_id", actor_id)
						.send()
						.await;

					if let Some(WorkflowError::WorkflowNotFound) = res.as_workflow_error() {
						// Try old actors
						let res = ctx
							.signal(crate::workflows::actor::v1::StateUpdate {
								generation,
								state: protocol::ActorState::Stopping,
							})
							.to_workflow::<crate::workflows::actor::v1::Workflow>()
							.tag("actor_id", actor_id)
							.send()
							.await;

						if let Some(WorkflowError::WorkflowNotFound) = res.as_workflow_error() {
							tracing::warn!(
								?actor_id,
								"actor workflow not found, likely already stopped"
							);
						} else {
							res?;
						}
					} else {
						res?;
					}
				}
			}
			protocol::Command::SignalRunner { .. } => {
				// No-op in this workflow
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
	let pool = ctx.sqlite().await?;
	let mut conn = pool.conn().await?;
	let mut tx = conn.begin().await?;

	let (last_command_index,) = sql_fetch_one!(
		[ctx, (i64,), @tx &mut tx]
		"
		UPDATE state
			SET last_command_idx = last_command_idx + ?
		RETURNING last_command_idx - ? + 1
		",
		input.commands.len() as i64,
		input.commands.len() as i64,
	)
	.await?;

	// TODO: Parallelize?
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
			last_command_index + index as i64,
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
	// Does not clear the data keys like last ping ts, just the allocation idx
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
struct FetchRemainingActorsInput {
	client_id: Uuid,
}

#[activity(FetchRemainingActors)]
async fn fetch_remaining_actors(
	ctx: &ActivityCtx,
	input: &FetchRemainingActorsInput,
) -> GlobalResult<Vec<(util::Id, u32)>> {
	let actor_ids = ctx
		.fdb()
		.await?
		.run(|tx, _mc| async move {
			let actor2_subspace =
				keys::subspace().subspace(&keys::client::Actor2Key::subspace(input.client_id));
			let actor_subspace =
				keys::subspace().subspace(&keys::client::ActorKey::subspace(input.client_id));

			tx.get_ranges_keyvalues(
				fdb::RangeOption {
					mode: StreamingMode::WantAll,
					..(&actor2_subspace).into()
				},
				SERIALIZABLE,
			)
			.chain(tx.get_ranges_keyvalues(
				fdb::RangeOption {
					mode: StreamingMode::WantAll,
					..(&actor_subspace).into()
				},
				SERIALIZABLE,
			))
			.map(|res| match res {
				Ok(entry) => {
					if let Ok(key) = keys::subspace().unpack::<keys::client::Actor2Key>(entry.key())
					{
						let generation = key
							.deserialize(entry.value())
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

						Ok((key.actor_id, generation))
					} else {
						let key = keys::subspace()
							.unpack::<keys::client::ActorKey>(entry.key())
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
						let generation = key
							.deserialize(entry.value())
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

						Ok((key.actor_id.into(), generation))
					}
				}
				Err(err) => Err(Into::<fdb::FdbBindingError>::into(err)),
			})
			.try_collect::<Vec<_>>()
			.await
		})
		.custom_instrument(tracing::info_span!("client_fetch_remaining_actors_tx"))
		.await?;

	Ok(actor_ids)
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct CheckExpiredInput {
	client_id: Uuid,
}

#[activity(CheckExpired)]
async fn check_expired(ctx: &ActivityCtx, input: &CheckExpiredInput) -> GlobalResult<bool> {
	let last_ping_ts = ctx
		.fdb()
		.await?
		.run(|tx, _mc| async move {
			let last_ping_ts_key = keys::client::LastPingTsKey::new(input.client_id);

			let last_ping_ts = tx
				.get(&keys::subspace().pack(&last_ping_ts_key), SERIALIZABLE)
				.await?;
			let last_ping_ts = last_ping_ts_key
				.deserialize(&last_ping_ts.ok_or(fdb::FdbBindingError::CustomError(
					format!("key should exist: {last_ping_ts_key:?}").into(),
				))?)
				.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

			Ok(last_ping_ts)
		})
		.custom_instrument(tracing::info_span!("client_check_expired_tx"))
		.await?;

	Ok(last_ping_ts < util::timestamp::now() - CLIENT_LOST_THRESHOLD_MS)
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub(crate) struct AllocatePendingActorsInput {}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AllocatePendingActorsOutput {
	pub allocations: Vec<ActorAllocation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ActorAllocation {
	pub actor_id: util::Id,
	pub signal: Allocate,
}

#[activity(AllocatePendingActors)]
pub(crate) async fn allocate_pending_actors(
	ctx: &ActivityCtx,
	input: &AllocatePendingActorsInput,
) -> GlobalResult<AllocatePendingActorsOutput> {
	let client_flavor = protocol::ClientFlavor::Multi;

	// NOTE: This txn should closely resemble the one found in the allocate_actor activity of the actor2 wf
	let res = ctx
		.fdb()
		.await?
		.run(|tx, _mc| async move {
			let mut results = Vec::new();

			let pending_actor_subspace =
				keys::subspace().subspace(&keys::datacenter::PendingActorKey::subspace());
			let mut queue_stream = tx.get_ranges_keyvalues(
				fdb::RangeOption {
					mode: StreamingMode::Iterator,
					..(&pending_actor_subspace).into()
				},
				// NOTE: This is not SERIALIZABLE because we don't want to conflict with all of the keys, just
				// the one we choose
				SNAPSHOT,
			);

			'queue_loop: loop {
				let Some(queue_entry) = queue_stream.try_next().await? else {
					break;
				};

				let queue_key = keys::subspace()
					.unpack::<keys::datacenter::PendingActorKey>(queue_entry.key())
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

				let queue_value = queue_key
					.deserialize(queue_entry.value())
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
				let memory_mib = queue_value.selected_mem / 1024 / 1024;

				// Check for availability amongst existing runners
				if let BuildRuntime::Actor { .. } = &queue_value.build_runtime {
					// Select a range that only includes runners that have enough remaining slots to allocate
					// this actor
					let start = keys::subspace().pack(
						&keys::datacenter::RunnersByRemainingSlotsKey::subspace_with_slots(
							queue_value.image_id,
							1,
						),
					);
					let runner_allocation_subspace =
						keys::datacenter::RunnersByRemainingSlotsKey::subspace(
							queue_value.image_id,
						);
					let end = keys::subspace()
						.subspace(&runner_allocation_subspace)
						.range()
						.1;

					// NOTE: This range read will include runners that were just inserted by the below code
					// because fdb supports read-your-writes by default. This is the behavior we want.
					let mut stream = tx.get_ranges_keyvalues(
						fdb::RangeOption {
							mode: StreamingMode::Iterator,
							// Containers bin pack so we reverse the order
							reverse: true,
							..(start, end).into()
						},
						// NOTE: This is not SERIALIZABLE because we don't want to conflict with all of the
						// keys, just the one we choose
						SNAPSHOT,
					);

					loop {
						let Some(entry) = stream.try_next().await? else {
							break;
						};

						let old_runner_allocation_key = keys::subspace()
							.unpack::<keys::datacenter::RunnersByRemainingSlotsKey>(entry.key())
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

						let data = old_runner_allocation_key
							.deserialize(entry.value())
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

						// Add read conflict only for this key
						tx.add_conflict_range(
							entry.key(),
							&end_of_key_range(entry.key()),
							ConflictRangeType::Read,
						)?;

						// Clear old entry
						tx.clear(entry.key());

						let new_remaining_slots =
							old_runner_allocation_key.remaining_slots.saturating_sub(1);

						// Write new allocation key with 1 less slot
						let new_allocation_key = keys::datacenter::RunnersByRemainingSlotsKey::new(
							queue_value.image_id,
							new_remaining_slots,
							old_runner_allocation_key.runner_id,
						);
						tx.set(&keys::subspace().pack(&new_allocation_key), entry.value());

						// Update runner record
						let remaining_slots_key = keys::runner::RemainingSlotsKey::new(
							old_runner_allocation_key.runner_id,
						);
						tx.set(
							&keys::subspace().pack(&remaining_slots_key),
							&remaining_slots_key
								.serialize(new_remaining_slots)
								.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
						);

						// Insert actor index key
						let client_actor_key =
							keys::client::Actor2Key::new(data.client_id, queue_key.actor_id);
						tx.set(
							&keys::subspace().pack(&client_actor_key),
							&client_actor_key
								.serialize(queue_value.generation)
								.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
						);

						// Add read conflict for the queue key
						tx.add_conflict_range(
							queue_entry.key(),
							&end_of_key_range(queue_entry.key()),
							ConflictRangeType::Read,
						)?;
						tx.clear(queue_entry.key());
						// Clear sister queue key
						tx.clear(&keys::subspace().pack(&queue_key.sister(queue_value.image_id)));

						results.push(ActorAllocation {
							actor_id: queue_key.actor_id,
							signal: Allocate {
								runner_id: old_runner_allocation_key.runner_id,
								new_runner: false,
								client_id: data.client_id,
								client_workflow_id: data.client_workflow_id,
							},
						});
						continue 'queue_loop;
					}
				}

				// No available runner found, create a new one

				let runner_id = Uuid::new_v4();

				let ping_threshold_ts = util::timestamp::now() - CLIENT_ELIGIBLE_THRESHOLD_MS;

				// Select a range that only includes clients that have enough remaining mem to allocate this
				// actor
				let start = keys::subspace().pack(
					&keys::datacenter::ClientsByRemainingMemKey::subspace_with_mem(
						client_flavor,
						memory_mib,
					),
				);
				let client_allocation_subspace =
					keys::datacenter::ClientsByRemainingMemKey::subspace(client_flavor);
				let end = keys::subspace()
					.subspace(&client_allocation_subspace)
					.range()
					.1;

				let mut stream = tx.get_ranges_keyvalues(
					fdb::RangeOption {
						mode: StreamingMode::Iterator,
						// Containers bin pack so we reverse the order
						reverse: true,
						..(start, end).into()
					},
					// NOTE: This is not SERIALIZABLE because we don't want to conflict with all of the keys,
					// just the one we choose
					SNAPSHOT,
				);

				loop {
					let Some(entry) = stream.try_next().await? else {
						break;
					};

					let old_client_allocation_key = keys::subspace()
						.unpack::<keys::datacenter::ClientsByRemainingMemKey>(entry.key())
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

					// Scan by last ping
					if old_client_allocation_key.last_ping_ts < ping_threshold_ts {
						continue;
					}

					let client_workflow_id =
						old_client_allocation_key
							.deserialize(entry.value())
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

					// Add read conflict only for this key
					tx.add_conflict_range(
						entry.key(),
						&end_of_key_range(entry.key()),
						ConflictRangeType::Read,
					)?;

					// Clear old entry
					tx.clear(entry.key());

					// Read old cpu
					let remaining_cpu_key =
						keys::client::RemainingCpuKey::new(old_client_allocation_key.client_id);
					let remaining_cpu_key_buf = keys::subspace().pack(&remaining_cpu_key);
					let remaining_cpu_entry = tx.get(&remaining_cpu_key_buf, SERIALIZABLE).await?;
					let old_remaining_cpu = remaining_cpu_key
						.deserialize(&remaining_cpu_entry.ok_or(
							fdb::FdbBindingError::CustomError(
								format!("key should exist: {remaining_cpu_key:?}").into(),
							),
						)?)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

					// Update allocated amount
					let new_remaining_mem = old_client_allocation_key.remaining_mem - memory_mib;
					let new_remaining_cpu = old_remaining_cpu - queue_value.selected_cpu;
					let new_allocation_key = keys::datacenter::ClientsByRemainingMemKey::new(
						client_flavor,
						new_remaining_mem,
						old_client_allocation_key.last_ping_ts,
						old_client_allocation_key.client_id,
					);
					tx.set(&keys::subspace().pack(&new_allocation_key), entry.value());

					tracing::debug!(
						old_mem=%old_client_allocation_key.remaining_mem,
						old_cpu=%old_remaining_cpu,
						new_mem=%new_remaining_mem,
						new_cpu=%new_remaining_cpu,
						"allocating runner resources"
					);

					// Update client record
					let remaining_mem_key =
						keys::client::RemainingMemoryKey::new(old_client_allocation_key.client_id);
					tx.set(
						&keys::subspace().pack(&remaining_mem_key),
						&remaining_mem_key
							.serialize(new_remaining_mem)
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
					);

					tx.set(
						&remaining_cpu_key_buf,
						&remaining_cpu_key
							.serialize(new_remaining_cpu)
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
					);

					let remaining_slots = queue_value.build_runtime.slots().saturating_sub(1);
					let total_slots = queue_value.build_runtime.slots();

					// Insert runner records
					let remaining_slots_key = keys::runner::RemainingSlotsKey::new(runner_id);
					tx.set(
						&keys::subspace().pack(&remaining_slots_key),
						&remaining_slots_key
							.serialize(remaining_slots)
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
					);

					let total_slots_key = keys::runner::TotalSlotsKey::new(runner_id);
					tx.set(
						&keys::subspace().pack(&total_slots_key),
						&total_slots_key
							.serialize(total_slots)
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
					);

					let image_id_key = keys::runner::ImageIdKey::new(runner_id);
					tx.set(
						&keys::subspace().pack(&image_id_key),
						&image_id_key
							.serialize(queue_value.image_id)
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
					);

					// Insert runner index key if actor. Container actor runners don't need to be in the alloc
					// idx because they only have 1 slot
					if let BuildRuntime::Actor { .. } = queue_value.build_runtime {
						let runner_idx_key = keys::datacenter::RunnersByRemainingSlotsKey::new(
							queue_value.image_id,
							remaining_slots,
							runner_id,
						);
						tx.set(
							&keys::subspace().pack(&runner_idx_key),
							&runner_idx_key
								.serialize(keys::datacenter::RunnersByRemainingSlotsKeyData {
									client_id: old_client_allocation_key.client_id,
									client_workflow_id,
								})
								.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
						);
					}

					// Insert actor index key
					let client_actor_key = keys::client::Actor2Key::new(
						old_client_allocation_key.client_id,
						queue_key.actor_id,
					);
					tx.set(
						&keys::subspace().pack(&client_actor_key),
						&client_actor_key
							.serialize(queue_value.generation)
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
					);

					// Add read conflict for the queue key
					tx.add_conflict_range(
						queue_entry.key(),
						&end_of_key_range(queue_entry.key()),
						ConflictRangeType::Read,
					)?;
					tx.clear(queue_entry.key());
					// Clear sister queue key
					tx.clear(&keys::subspace().pack(&queue_key.sister(queue_value.image_id)));

					results.push(ActorAllocation {
						actor_id: queue_key.actor_id,
						signal: Allocate {
							runner_id,
							new_runner: true,
							client_id: old_client_allocation_key.client_id,
							client_workflow_id,
						},
					});

					continue 'queue_loop;
				}
			}

			Ok(results)
		})
		.custom_instrument(tracing::info_span!("client_allocate_pending_actors_tx"))
		.await?;

	Ok(AllocatePendingActorsOutput { allocations: res })
}

// TODO: This is called fairly frequently
#[derive(Debug, Serialize, Deserialize, Hash)]
struct UpdateMetricsInput {
	client_id: Uuid,
	flavor: ClientFlavor,
	#[serde(default)]
	draining: bool,
	clear: bool,
}

#[activity(UpdateMetrics)]
async fn update_metrics(ctx: &ActivityCtx, input: &UpdateMetricsInput) -> GlobalResult<()> {
	if input.clear {
		metrics::CLIENT_MEMORY_TOTAL
			.with_label_values(&[
				&input.client_id.to_string(),
				&input.flavor.to_string(),
				"active",
			])
			.set(0);
		metrics::CLIENT_CPU_TOTAL
			.with_label_values(&[
				&input.client_id.to_string(),
				&input.flavor.to_string(),
				"active",
			])
			.set(0);
		metrics::CLIENT_MEMORY_TOTAL
			.with_label_values(&[
				&input.client_id.to_string(),
				&input.flavor.to_string(),
				"draining",
			])
			.set(0);
		metrics::CLIENT_CPU_TOTAL
			.with_label_values(&[
				&input.client_id.to_string(),
				&input.flavor.to_string(),
				"draining",
			])
			.set(0);
		metrics::CLIENT_MEMORY_ALLOCATED
			.with_label_values(&[
				&input.client_id.to_string(),
				&input.flavor.to_string(),
				"active",
			])
			.set(0);
		metrics::CLIENT_CPU_ALLOCATED
			.with_label_values(&[
				&input.client_id.to_string(),
				&input.flavor.to_string(),
				"active",
			])
			.set(0);
		metrics::CLIENT_MEMORY_ALLOCATED
			.with_label_values(&[
				&input.client_id.to_string(),
				&input.flavor.to_string(),
				"draining",
			])
			.set(0);
		metrics::CLIENT_CPU_ALLOCATED
			.with_label_values(&[
				&input.client_id.to_string(),
				&input.flavor.to_string(),
				"draining",
			])
			.set(0);

		return Ok(());
	}

	let (total_mem, remaining_mem, total_cpu, remaining_cpu) =
		ctx.fdb()
			.await?
			.run(|tx, _mc| async move {
				let total_mem_key = keys::client::TotalMemoryKey::new(input.client_id);
				let remaining_mem_key = keys::client::RemainingMemoryKey::new(input.client_id);
				let total_cpu_key = keys::client::TotalCpuKey::new(input.client_id);
				let remaining_cpu_key = keys::client::RemainingCpuKey::new(input.client_id);

				let (total_mem_entry, remaining_mem_entry, total_cpu_entry, remaining_cpu_entry) =
					tokio::try_join!(
						tx.get(&keys::subspace().pack(&total_mem_key), SNAPSHOT),
						tx.get(&keys::subspace().pack(&remaining_mem_key), SNAPSHOT),
						tx.get(&keys::subspace().pack(&total_cpu_key), SNAPSHOT),
						tx.get(&keys::subspace().pack(&remaining_cpu_key), SNAPSHOT),
					)?;

				let total_mem = total_mem_key
					.deserialize(&total_mem_entry.ok_or(fdb::FdbBindingError::CustomError(
						format!("key should exist: {total_mem_key:?}").into(),
					))?)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
				let remaining_mem = remaining_mem_key
					.deserialize(
						&remaining_mem_entry.ok_or(fdb::FdbBindingError::CustomError(
							format!("key should exist: {remaining_mem_key:?}").into(),
						))?,
					)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
				let total_cpu = total_cpu_key
					.deserialize(&total_cpu_entry.ok_or(fdb::FdbBindingError::CustomError(
						format!("key should exist: {total_cpu_key:?}").into(),
					))?)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
				let remaining_cpu = remaining_cpu_key
					.deserialize(
						&remaining_cpu_entry.ok_or(fdb::FdbBindingError::CustomError(
							format!("key should exist: {remaining_cpu_key:?}").into(),
						))?,
					)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

				Ok((total_mem, remaining_mem, total_cpu, remaining_cpu))
			})
			.custom_instrument(tracing::info_span!("client_update_metrics_tx"))
			.await?;

	let (state, other_state) = if input.draining {
		("draining", "active")
	} else {
		("active", "draining")
	};
	let allocated_mem = total_mem.saturating_sub(remaining_mem);
	let allocated_cpu = total_cpu.saturating_sub(remaining_cpu);

	metrics::CLIENT_MEMORY_TOTAL
		.with_label_values(&[
			&input.client_id.to_string(),
			&input.flavor.to_string(),
			state,
		])
		.set(total_mem.try_into()?);
	metrics::CLIENT_CPU_TOTAL
		.with_label_values(&[
			&input.client_id.to_string(),
			&input.flavor.to_string(),
			state,
		])
		.set(total_cpu.try_into()?);

	metrics::CLIENT_MEMORY_ALLOCATED
		.with_label_values(&[
			&input.client_id.to_string(),
			&input.flavor.to_string(),
			state,
		])
		.set(allocated_mem.try_into()?);
	metrics::CLIENT_CPU_ALLOCATED
		.with_label_values(&[
			&input.client_id.to_string(),
			&input.flavor.to_string(),
			state,
		])
		.set(allocated_cpu.try_into()?);

	// Clear other state
	metrics::CLIENT_MEMORY_TOTAL
		.with_label_values(&[
			&input.client_id.to_string(),
			&input.flavor.to_string(),
			other_state,
		])
		.set(0);
	metrics::CLIENT_CPU_TOTAL
		.with_label_values(&[
			&input.client_id.to_string(),
			&input.flavor.to_string(),
			other_state,
		])
		.set(0);

	metrics::CLIENT_MEMORY_ALLOCATED
		.with_label_values(&[
			&input.client_id.to_string(),
			&input.flavor.to_string(),
			other_state,
		])
		.set(0);
	metrics::CLIENT_CPU_ALLOCATED
		.with_label_values(&[
			&input.client_id.to_string(),
			&input.flavor.to_string(),
			other_state,
		])
		.set(0);

	Ok(())
}

#[message("pegboard_client_to_ws")]
pub struct ToWs {
	pub client_id: Uuid,
	pub inner: protocol::ToClient,
}

#[signal("pegboard_prewarm_image2")]
pub struct PrewarmImage2 {
	pub image: protocol::Image,
}

#[signal("pegboard_client_check_queue")]
pub struct CheckQueue {}

#[message("pegboard_client_close_ws")]
pub struct CloseWs {
	pub client_id: Uuid,
}

#[signal("pegboard_client_drain")]
pub struct Drain {
	pub drain_timeout_ts: i64,
}

#[signal("pegboard_client_undrain")]
pub struct Undrain {}

join_signal!(Main {
	Command(protocol::Command),
	// Forwarded from the ws to this workflow
	Forward(protocol::ToServer),
	PrewarmImage2,
	CheckQueue,
	Drain,
	Undrain,
});
