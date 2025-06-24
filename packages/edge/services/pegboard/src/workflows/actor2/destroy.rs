use build::types::BuildAllocationType;
use chirp_workflow::prelude::*;
use fdb_util::{end_of_key_range, FormalKey, SERIALIZABLE};
use foundationdb::{self as fdb, options::ConflictRangeType};
use nix::sys::signal::Signal;

use super::{
	analytics::InsertClickHouseInput, runtime::ActorRunnerClickhouseRow, DestroyComplete,
	DestroyStarted,
};
use crate::{keys, protocol, types::GameGuardProtocol};

#[derive(Debug, Serialize, Deserialize)]
pub struct KillCtx {
	pub kill_timeout_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Input {
	pub actor_id: util::Id,
	pub generation: u32,
	pub image_id: Uuid,
	pub build_allocation_type: Option<BuildAllocationType>,
	/// Whether or not to send signals to the pb actor. In the case that the actor was already stopped
	/// or exited, signals are unnecessary.
	pub kill: Option<KillCtx>,
}

#[workflow]
pub(crate) async fn pegboard_actor_destroy(
	ctx: &mut WorkflowCtx,
	input: &Input,
) -> GlobalResult<()> {
	ctx.msg(DestroyStarted {})
		.tag("actor_id", input.actor_id)
		.send()
		.await?;

	let actor = ctx.activity(UpdateDbInput {}).await?;

	if let Some(actor) = actor {
		if let (Some(start_ts), Some(runner_id)) = (actor.start_ts, actor.runner_id) {
			ctx.activity(FinishRunnerClickhouseInput {
				actor_id: input.actor_id,
				generation: input.generation,
				start_ts,
				runner_id,
			})
			.await?;
		}

		let client_workflow_id = actor.client_workflow_id;
		let runner_id = actor.runner_id;

		let res = ctx
			.activity(UpdateFdbInput {
				actor_id: input.actor_id,
				image_id: input.image_id,
				build_allocation_type: input.build_allocation_type,
				actor,
			})
			.await?;

		// Destroy actor
		if let (Some(client_workflow_id), Some(kill_data)) = (client_workflow_id, &input.kill) {
			kill(
				ctx,
				input.actor_id,
				input.generation,
				client_workflow_id,
				kill_data.kill_timeout_ms,
				false,
			)
			.await?;
		}

		// Destroy runner
		if let (Some(client_workflow_id), Some(runner_id), true) =
			(client_workflow_id, runner_id, res.destroy_runner)
		{
			ctx.signal(protocol::Command::SignalRunner {
				runner_id,
				signal: Signal::SIGKILL as i32,
			})
			.to_workflow_id(client_workflow_id)
			.send()
			.await?;
		}
	}

	// Update ClickHouse analytics with destroyed timestamp
	ctx.v(2)
		.activity(InsertClickHouseInput {
			actor_id: input.actor_id,
		})
		.await?;

	ctx.msg(DestroyComplete {})
		.tag("actor_id", input.actor_id)
		.send()
		.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct UpdateDbInput {}

#[derive(Debug, Serialize, Deserialize, Hash, sqlx::FromRow)]
struct UpdateDbOutput {
	env_id: Uuid,
	selected_resources_memory_mib: Option<i64>,
	selected_resources_cpu_millicores: Option<i64>,
	tags: sqlx::types::Json<util::serde::HashableMap<String, String>>,
	create_ts: i64,
	start_ts: Option<i64>,
	runner_id: Option<Uuid>,
	client_id: Option<Uuid>,
	client_workflow_id: Option<Uuid>,
}

#[activity(UpdateDb)]
async fn update_db(
	ctx: &ActivityCtx,
	input: &UpdateDbInput,
) -> GlobalResult<Option<UpdateDbOutput>> {
	let pool = ctx.sqlite().await?;

	// NOTE: Row might not exist if the workflow failed before insert_db
	sql_fetch_optional!(
		[ctx, UpdateDbOutput, pool]
		"
		UPDATE state
		SET destroy_ts = ?
		RETURNING
			env_id,
			selected_resources_memory_mib,
			selected_resources_cpu_millicores,
			json(tags) AS tags,
			create_ts,
			start_ts,
			runner_id,
			client_id,
			client_workflow_id
		",
		ctx.ts(),
	)
	.await
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct FinishRunnerClickhouseInput {
	actor_id: util::Id,
	generation: u32,
	start_ts: i64,
	runner_id: Uuid,
}

#[activity(FinishRunnerClickhouse)]
async fn finish_runner_clickhouse(
	ctx: &ActivityCtx,
	input: &FinishRunnerClickhouseInput,
) -> GlobalResult<()> {
	let inserter = ctx.clickhouse_inserter().await?;

	// Set alloc as finished
	inserter.insert(
		"db_pegboard_runner",
		"actor_runners",
		ActorRunnerClickhouseRow {
			actor_id: input.actor_id.to_string(),
			generation: input.generation,
			runner_id: input.runner_id,
			started_at: input.start_ts * 1_000_000, // Convert ms to ns for ClickHouse DateTime64(9)
			finished_at: util::timestamp::now() * 1_000_000, // Convert ms to ns for ClickHouse DateTime64(9)
		},
	)?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct UpdateFdbInput {
	actor_id: util::Id,
	image_id: Uuid,
	build_allocation_type: Option<BuildAllocationType>,
	actor: UpdateDbOutput,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct UpdateFdbOutput {
	destroy_runner: bool,
}

#[activity(UpdateFdb)]
pub async fn update_fdb(
	ctx: &ActivityCtx,
	input: &UpdateFdbInput,
) -> GlobalResult<UpdateFdbOutput> {
	let pool = ctx.sqlite().await?;

	let ingress_ports = sql_fetch_all!(
		[ctx, (i64, i64), pool]
		"
		SELECT protocol, ingress_port_number
		FROM ports_ingress
		",
	)
	.await?;

	let destroy_runner = ctx
		.fdb()
		.await?
		.run(|tx, _mc| {
			let ingress_ports = ingress_ports.clone();
			async move {
				// Update actor key index in env subspace
				let actor_key = keys::env::Actor2Key::new(
					input.actor.env_id,
					input.actor.create_ts,
					input.actor_id,
				);
				let data = keys::env::Actor2KeyData {
					is_destroyed: true,
					tags: input.actor.tags.0.clone().into_iter().collect(),
				};
				tx.set(
					&keys::subspace().pack(&actor_key),
					&actor_key
						.serialize(data)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);

				clear_ports_and_resources(
					input.actor_id,
					input.image_id,
					input.build_allocation_type,
					ingress_ports,
					input.actor.runner_id,
					input.actor.client_id,
					input.actor.client_workflow_id,
					input.actor.selected_resources_memory_mib,
					input.actor.selected_resources_cpu_millicores,
					&tx,
				)
				.await
			}
		})
		.custom_instrument(tracing::info_span!("actor_destroy_tx"))
		.await?;

	Ok(UpdateFdbOutput { destroy_runner })
}

// TODO: Clean up args
/// Clears allocated ports and resources (if they were allocated).
pub(crate) async fn clear_ports_and_resources(
	actor_id: util::Id,
	image_id: Uuid,
	build_allocation_type: Option<BuildAllocationType>,
	ingress_ports: Vec<(i64, i64)>,
	runner_id: Option<Uuid>,
	client_id: Option<Uuid>,
	client_workflow_id: Option<Uuid>,
	selected_resources_memory_mib: Option<i64>,
	selected_resources_cpu_millicores: Option<i64>,
	tx: &fdb::RetryableTransaction,
) -> Result<bool, fdb::FdbBindingError> {
	// Remove all allocated ingress ports
	for (protocol, port) in ingress_ports {
		let ingress_port_key = keys::port::IngressKey2::new(
			GameGuardProtocol::from_repr(
				usize::try_from(protocol)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			)
			.ok_or_else(|| {
				fdb::FdbBindingError::CustomError(
					format!("invalid protocol variant: {protocol}").into(),
				)
			})?,
			u16::try_from(port).map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			actor_id,
		);

		tx.clear(&keys::subspace().pack(&ingress_port_key));
	}

	// Remove proxied ports
	let proxied_ports_key = keys::actor2::ProxiedPortsKey::new(actor_id);
	tx.clear(&keys::subspace().pack(&proxied_ports_key));

	if let Some(client_id) = client_id {
		// This is cleared when the state changes as well as when the actor is destroyed to ensure
		// consistency during rescheduling and forced deletion.
		let actor_key = keys::client::Actor2Key::new(client_id, actor_id);
		tx.clear(&keys::subspace().pack(&actor_key));
	}

	// Release client's resources and update allocation index
	if let (
		Some(build_allocation_type),
		Some(runner_id),
		Some(client_id),
		Some(client_workflow_id),
		Some(selected_resources_memory_mib),
		Some(selected_resources_cpu_millicores),
	) = (
		build_allocation_type,
		runner_id,
		client_id,
		client_workflow_id,
		selected_resources_memory_mib,
		selected_resources_cpu_millicores,
	) {
		let client_flavor = protocol::ClientFlavor::Multi;

		let runner_remaining_slots_key = keys::runner::RemainingSlotsKey::new(runner_id);
		let runner_remaining_slots_key_buf = keys::subspace().pack(&runner_remaining_slots_key);
		let runner_total_slots_key = keys::runner::TotalSlotsKey::new(runner_id);
		let runner_total_slots_key_buf = keys::subspace().pack(&runner_total_slots_key);
		let client_remaining_mem_key = keys::client::RemainingMemoryKey::new(client_id);
		let client_remaining_mem_key_buf = keys::subspace().pack(&client_remaining_mem_key);
		let client_remaining_cpu_key = keys::client::RemainingCpuKey::new(client_id);
		let client_remaining_cpu_key_buf = keys::subspace().pack(&client_remaining_cpu_key);
		let client_last_ping_ts_key = keys::client::LastPingTsKey::new(client_id);
		let client_last_ping_ts_key_buf = keys::subspace().pack(&client_last_ping_ts_key);

		let (
			runner_remaining_slots_entry,
			runner_total_slots_entry,
			client_remaining_mem_entry,
			client_remaining_cpu_entry,
			client_last_ping_ts_entry,
		) = tokio::try_join!(
			tx.get(&runner_remaining_slots_key_buf, SERIALIZABLE),
			tx.get(&runner_total_slots_key_buf, SERIALIZABLE),
			tx.get(&client_remaining_mem_key_buf, SERIALIZABLE),
			tx.get(&client_remaining_cpu_key_buf, SERIALIZABLE),
			tx.get(&client_last_ping_ts_key_buf, SERIALIZABLE),
		)?;

		let runner_remaining_slots = runner_remaining_slots_key
			.deserialize(
				&runner_remaining_slots_entry.ok_or(fdb::FdbBindingError::CustomError(
					format!("key should exist: {runner_remaining_slots_key:?}").into(),
				))?,
			)
			.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
		let runner_total_slots = runner_total_slots_key
			.deserialize(
				&runner_total_slots_entry.ok_or(fdb::FdbBindingError::CustomError(
					format!("key should exist: {runner_total_slots_key:?}").into(),
				))?,
			)
			.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
		let client_remaining_mem = client_remaining_mem_key
			.deserialize(
				&client_remaining_mem_entry.ok_or(fdb::FdbBindingError::CustomError(
					format!("key should exist: {client_remaining_mem_key:?}").into(),
				))?,
			)
			.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
		let client_remaining_cpu = client_remaining_cpu_key
			.deserialize(
				&client_remaining_cpu_entry.ok_or(fdb::FdbBindingError::CustomError(
					format!("key should exist: {client_remaining_cpu_key:?}").into(),
				))?,
			)
			.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
		let client_last_ping_ts = client_last_ping_ts_key
			.deserialize(
				&client_last_ping_ts_entry.ok_or(fdb::FdbBindingError::CustomError(
					format!("key should exist: {client_last_ping_ts_key:?}").into(),
				))?,
			)
			.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

		let old_runner_allocation_key = keys::datacenter::RunnersByRemainingSlotsKey::new(
			image_id,
			runner_remaining_slots,
			runner_id,
		);
		let old_runner_allocation_key_buf = keys::subspace().pack(&old_runner_allocation_key);

		let old_client_allocation_key = keys::datacenter::ClientsByRemainingMemKey::new(
			client_flavor,
			client_remaining_mem,
			client_last_ping_ts,
			client_id,
		);
		let old_client_allocation_key_buf = keys::subspace().pack(&old_client_allocation_key);

		let new_runner_remaining_slots = runner_remaining_slots + 1;

		// Write new remaining slots
		tx.set(
			&runner_remaining_slots_key_buf,
			&runner_remaining_slots_key
				.serialize(new_runner_remaining_slots)
				.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
		);

		// Clear old key
		tx.clear(&old_runner_allocation_key_buf);

		// Add read conflict
		tx.add_conflict_range(
			&old_runner_allocation_key_buf,
			&end_of_key_range(&old_runner_allocation_key_buf),
			ConflictRangeType::Read,
		)?;

		let destroy_runner = if new_runner_remaining_slots < runner_total_slots {
			let new_runner_allocation_key = keys::datacenter::RunnersByRemainingSlotsKey::new(
				image_id,
				new_runner_remaining_slots,
				runner_id,
			);
			let new_runner_allocation_key_buf = keys::subspace().pack(&new_runner_allocation_key);

			tx.set(
				&new_runner_allocation_key_buf,
				&new_runner_allocation_key
					.serialize(keys::datacenter::RunnersByRemainingSlotsKeyData {
						client_id,
						client_workflow_id,
					})
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			);

			false
		}
		// Runner is now empty, release client resources
		else {
			let new_client_remaining_mem = client_remaining_mem
				+ u64::try_from(selected_resources_memory_mib)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;
			let new_client_remaining_cpu = client_remaining_cpu
				+ u64::try_from(selected_resources_cpu_millicores)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?;

			tracing::debug!(
				old_mem=%client_remaining_mem,
				old_cpu=%client_remaining_cpu,
				new_mem=%new_client_remaining_mem,
				new_cpu=%new_client_remaining_cpu,
				"releasing resources"
			);

			// Write new memory
			tx.set(
				&client_remaining_mem_key_buf,
				&client_remaining_mem_key
					.serialize(new_client_remaining_mem)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			);
			// Write new cpu
			tx.set(
				&client_remaining_cpu_key_buf,
				&client_remaining_cpu_key
					.serialize(new_client_remaining_cpu)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			);

			// Only update allocation idx if it existed before
			if tx
				.get(&old_client_allocation_key_buf, SERIALIZABLE)
				.await?
				.is_some()
			{
				// Clear old key
				tx.clear(&old_client_allocation_key_buf);

				let new_client_allocation_key = keys::datacenter::ClientsByRemainingMemKey::new(
					client_flavor,
					new_client_remaining_mem,
					client_last_ping_ts,
					client_id,
				);
				let new_client_allocation_key_buf =
					keys::subspace().pack(&new_client_allocation_key);

				tx.set(
					&new_client_allocation_key_buf,
					&new_client_allocation_key
						.serialize(client_workflow_id)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);
			}

			// Single container per runner allocations don't require explicitly destroying the runner because
			// it is already stopped; the container = the actor.
			matches!(build_allocation_type, BuildAllocationType::Multi)
		};

		Ok(destroy_runner)
	} else {
		Ok(false)
	}
}

pub(crate) async fn kill(
	ctx: &mut WorkflowCtx,
	actor_id: util::Id,
	generation: u32,
	client_workflow_id: Uuid,
	kill_timeout_ms: i64,
	persist_storage: bool,
) -> GlobalResult<()> {
	if kill_timeout_ms != 0 {
		ctx.signal(protocol::Command::SignalActor {
			actor_id,
			generation,
			signal: Signal::SIGTERM as i32,
			persist_storage,
		})
		.to_workflow_id(client_workflow_id)
		.send()
		.await?;

		// See `docs/packages/job/JOB_DRAINING_AND_KILL_TIMEOUTS.md`
		ctx.sleep(kill_timeout_ms).await?;
	}

	ctx.signal(protocol::Command::SignalActor {
		actor_id,
		generation,
		signal: Signal::SIGKILL as i32,
		persist_storage,
	})
	.to_workflow_id(client_workflow_id)
	.send()
	.await?;

	Ok(())
}
