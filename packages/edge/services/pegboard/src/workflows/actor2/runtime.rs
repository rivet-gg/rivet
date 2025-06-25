use std::time::Instant;

use build::types::{BuildAllocationType, BuildKind};
use chirp_workflow::prelude::*;
use cluster::types::BuildDeliveryMethod;
use fdb_util::{end_of_key_range, FormalKey, SERIALIZABLE, SNAPSHOT};
use foundationdb::{
	self as fdb,
	options::{ConflictRangeType, StreamingMode},
};
use futures_util::StreamExt;
use futures_util::{FutureExt, TryStreamExt};
use nix::sys::signal::Signal;
use sqlx::Acquire;

use super::{
	destroy::{self, KillCtx},
	setup, Allocate, Destroy, Input, PendingAllocation, ACTOR_START_THRESHOLD_MS,
	BASE_RETRY_TIMEOUT_MS, RETRY_RESET_DURATION_MS,
};
use crate::{
	keys, metrics,
	ops::actor::get,
	protocol,
	types::{EndpointType, GameGuardProtocol, HostProtocol, NetworkMode, Port, Routing},
	workflows::client::CLIENT_ELIGIBLE_THRESHOLD_MS,
};

#[derive(Deserialize, Serialize)]
pub struct State {
	pub generation: u32,
	pub runner_id: Uuid,

	pub client_id: Uuid,
	pub client_workflow_id: Uuid,
	pub image_id: Uuid,

	pub drain_timeout_ts: Option<i64>,
	pub gc_timeout_ts: Option<i64>,

	#[serde(default)]
	reschedule_state: RescheduleState,
}

impl State {
	pub fn new(runner_id: Uuid, client_id: Uuid, client_workflow_id: Uuid, image_id: Uuid) -> Self {
		State {
			generation: 0,
			client_id,
			client_workflow_id,
			runner_id,
			image_id,
			drain_timeout_ts: None,
			gc_timeout_ts: Some(util::timestamp::now() + ACTOR_START_THRESHOLD_MS),
			reschedule_state: RescheduleState::default(),
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct LifecycleRes {
	pub generation: u32,
	pub image_id: Uuid,
	pub kill: Option<KillCtx>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct RescheduleState {
	last_retry_ts: i64,
	retry_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct UpdateClientAndRunnerInput {
	client_id: Uuid,
	client_workflow_id: Uuid,
	runner_id: Uuid,
}

#[activity(UpdateClientAndRunner)]
async fn update_client_and_runner(
	ctx: &ActivityCtx,
	input: &UpdateClientAndRunnerInput,
) -> GlobalResult<()> {
	let client_pool = ctx.sqlite_for_workflow(input.client_workflow_id).await?;
	let pool = ctx.sqlite().await?;

	let (client_wan_hostname,) = sql_fetch_one!(
		[ctx, (String,), client_pool]
		"
		SELECT config->'network'->>'wan_hostname' AS wan_hostname
		FROM state
		",
	)
	.await?;

	sql_execute!(
		[ctx, &pool]
		"
		UPDATE state
		SET
			pending_allocation_ts = NULL,
			client_id = ?1,
			client_workflow_id = ?2,
			client_wan_hostname = ?3,
			runner_id = ?4,
			old_runner_id = runner_id
		",
		input.client_id,
		input.client_workflow_id,
		&client_wan_hostname,
		input.runner_id,
	)
	.await?;

	Ok(())
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

#[derive(Debug, Serialize, Deserialize, Hash)]
struct FetchPortsInput {
	actor_id: util::Id,
	endpoint_type: Option<EndpointType>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FetchPortsOutput {
	ports: Vec<FetchedPort>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FetchedPort {
	name: String,
	port_number: Option<u16>,
	port: Port,
}

#[activity(FetchPorts)]
async fn fetch_ports(ctx: &ActivityCtx, input: &FetchPortsInput) -> GlobalResult<FetchPortsOutput> {
	let pool = ctx.sqlite().await?;

	let dc_id = ctx.config().server()?.rivet.edge()?.datacenter_id;

	let ((wan_hostname,), port_ingress_rows, port_host_rows, dc_res) = tokio::try_join!(
		sql_fetch_one!(
			[ctx, (Option<String>,), &pool]
			"
			SELECT client_wan_hostname
			FROM state
			",
		),
		sql_fetch_all!(
			[ctx, get::PortIngress, &pool]
			"
			SELECT
				port_name,
				port_number,
				ingress_port_number,
				protocol
			FROM ports_ingress
			",
		),
		sql_fetch_all!(
			[ctx, get::PortHost, &pool]
			"
			SELECT port_name, port_number, protocol
			FROM ports_host
			",
		),
		ctx.op(cluster::ops::datacenter::get::Input {
			datacenter_ids: vec![dc_id],
		}),
	)?;

	let dc = unwrap!(dc_res.datacenters.first());

	let endpoint_type = input.endpoint_type.unwrap_or_else(|| {
		EndpointType::default_for_guard_public_hostname(&dc.guard_public_hostname)
	});

	let ports = port_ingress_rows
		.into_iter()
		.map(|row| {
			let port = get::create_port_ingress(
				input.actor_id,
				&row,
				unwrap!(GameGuardProtocol::from_repr(row.protocol.try_into()?)),
				endpoint_type,
				&dc.guard_public_hostname,
			)?;

			Ok(FetchedPort {
				name: row.port_name,
				port_number: row.port_number.map(TryInto::try_into).transpose()?,
				port,
			})
		})
		.chain(port_host_rows.into_iter().map(|row| {
			let port = get::create_port_host(
				true,
				wan_hostname.as_deref(),
				&row,
				// Placeholder, will be replaced by the manager when building metadata
				Some(&get::PortProxied {
					port_name: String::new(),
					source: 0,
				}),
			)?;

			Ok(FetchedPort {
				name: row.port_name,
				port_number: row.port_number.map(TryInto::try_into).transpose()?,
				port,
			})
		}))
		.collect::<GlobalResult<Vec<_>>>()?;

	Ok(FetchPortsOutput { ports })
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct AllocateActorInput {
	actor_id: util::Id,
	generation: u32,
	image_id: Uuid,
	build_allocation_type: BuildAllocationType,
	build_allocation_total_slots: u32,
	resources: protocol::Resources,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllocateActorOutput {
	pub runner_id: Uuid,
	pub new_runner: bool,
	pub client_id: Uuid,
	pub client_workflow_id: Uuid,
}

// If no availability, returns the timestamp of the actor's queue key
#[activity(AllocateActor)]
async fn allocate_actor(
	ctx: &ActivityCtx,
	input: &AllocateActorInput,
) -> GlobalResult<Result<AllocateActorOutput, i64>> {
	let client_flavor = protocol::ClientFlavor::Multi;
	let memory_mib = input.resources.memory / 1024 / 1024;

	let start_instant = Instant::now();

	// NOTE: This txn should closely resemble the one found in the allocate_pending_actors activity of the
	// client wf
	let res = ctx
		.fdb()
		.await?
		.run(|tx, _mc| async move {
			// Check for availability amongst existing runners
			let image_queue_exists = if let BuildAllocationType::Multi = input.build_allocation_type
			{
				// Check if a queue for this image exists
				let pending_actor_by_image_subspace = keys::subspace().subspace(
					&keys::datacenter::PendingActorByImageIdKey::subspace(input.image_id),
				);
				let queue_exists = tx
					.get_ranges_keyvalues(
						fdb::RangeOption {
							mode: StreamingMode::Exact,
							limit: Some(1),
							..(&pending_actor_by_image_subspace).into()
						},
						// NOTE: This is not SERIALIZABLE because we don't want to conflict with other
						// inserts/clears to this range
						// queue
						SNAPSHOT,
					)
					.try_next()
					.await?
					.is_some();

				if !queue_exists {
					// Select a range that only includes runners that have enough remaining slots to allocate
					// this actor
					let start = keys::subspace().pack(
						&keys::datacenter::RunnersByRemainingSlotsKey::subspace_with_slots(
							input.image_id,
							1,
						),
					);
					let runner_allocation_subspace =
						keys::datacenter::RunnersByRemainingSlotsKey::subspace(input.image_id);
					let end = keys::subspace()
						.subspace(&runner_allocation_subspace)
						.range()
						.1;

					let mut stream = tx.get_ranges_keyvalues(
						fdb::RangeOption {
							mode: StreamingMode::Iterator,
							// Containers bin pack so we reverse the order
							reverse: true,
							..(start, end).into()
						},
						// NOTE: This is not SERIALIZABLE because we don't want to conflict with all of the keys, just
						// the one we choose
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
							input.image_id,
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
							keys::client::Actor2Key::new(data.client_id, input.actor_id);
						tx.set(
							&keys::subspace().pack(&client_actor_key),
							&client_actor_key
								.serialize(input.generation)
								.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
						);

						return Ok(Ok(AllocateActorOutput {
							runner_id: old_runner_allocation_key.runner_id,
							new_runner: false,
							client_id: data.client_id,
							client_workflow_id: data.client_workflow_id,
						}));
					}
				}

				queue_exists
			} else {
				false
			};

			// No available runner found, create a new one

			// Check if a queue exists
			let pending_actor_subspace =
				keys::subspace().subspace(&keys::datacenter::PendingActorKey::subspace());
			let queue_exists = if image_queue_exists {
				// We don't have to check the range if the image queue exists, its guaranteed that this one
				// exists too
				true
			} else {
				tx.get_ranges_keyvalues(
					fdb::RangeOption {
						mode: StreamingMode::Exact,
						limit: Some(1),
						..(&pending_actor_subspace).into()
					},
					// NOTE: This is not SERIALIZABLE because we don't want to conflict with other
					// inserts/clears to this range
					// queue
					SNAPSHOT,
				)
				.next()
				.await
				.is_some()
			};

			if !queue_exists {
				let runner_id = Uuid::new_v4();

				let ping_threshold_ts = util::timestamp::now() - CLIENT_ELIGIBLE_THRESHOLD_MS;

				// Select a range that only includes clients that have enough remaining mem to allocate this actor
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
					// NOTE: This is not SERIALIZABLE because we don't want to conflict with all of the keys, just
					// the one we choose
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
					let new_remaining_cpu = old_remaining_cpu - input.resources.cpu;
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

					let remaining_slots = input.build_allocation_total_slots.saturating_sub(1);
					let total_slots = input.build_allocation_total_slots;

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
							.serialize(input.image_id)
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
					);

					// Insert runner index key if multi. Single allocation per container runners don't need to be
					// in the alloc idx because they only have 1 slot
					if let BuildAllocationType::Multi = input.build_allocation_type {
						let runner_idx_key = keys::datacenter::RunnersByRemainingSlotsKey::new(
							input.image_id,
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
						input.actor_id,
					);
					tx.set(
						&keys::subspace().pack(&client_actor_key),
						&client_actor_key
							.serialize(input.generation)
							.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
					);

					return Ok(Ok(AllocateActorOutput {
						runner_id,
						new_runner: true,
						client_id: old_client_allocation_key.client_id,
						client_workflow_id,
					}));
				}
			}

			// At this point in the txn there is no availability. Write the actor to the alloc queue to wait.

			let pending_ts = util::timestamp::now();

			// Write self to image alloc queue
			if let BuildAllocationType::Multi = input.build_allocation_type {
				let image_pending_alloc_key = keys::datacenter::PendingActorByImageIdKey::new(
					input.image_id,
					pending_ts,
					input.actor_id,
				);
				let image_pending_alloc_data = keys::datacenter::PendingActorByImageIdKeyData {
					generation: input.generation,
					build_allocation_type: input.build_allocation_type,
					build_allocation_total_slots: input.build_allocation_total_slots,
					cpu: input.resources.cpu,
					memory: input.resources.memory,
				};

				// NOTE: This will conflict with serializable reads to the alloc queue, which is the behavior we
				// want. If a client reads from the queue while this is being inserted, one of the two txns will
				// retry and we ensure the actor does not end up in queue limbo.
				tx.set(
					&keys::subspace().pack(&image_pending_alloc_key),
					&image_pending_alloc_key
						.serialize(image_pending_alloc_data)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);
			}

			// Write self to global alloc queue
			let pending_alloc_key =
				keys::datacenter::PendingActorKey::new(pending_ts, input.actor_id);
			let pending_alloc_data = keys::datacenter::PendingActorKeyData {
				generation: input.generation,
				image_id: input.image_id,
				build_allocation_type: input.build_allocation_type,
				build_allocation_total_slots: input.build_allocation_total_slots,
				cpu: input.resources.cpu,
				memory: input.resources.memory,
			};

			// NOTE: This will conflict with serializable reads to the alloc queue, which is the behavior we
			// want. If a client reads from the queue while this is being inserted, one of the two txns will
			// retry and we ensure the actor does not end up in queue limbo.
			tx.set(
				&keys::subspace().pack(&pending_alloc_key),
				&pending_alloc_key
					.serialize(pending_alloc_data)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			);

			return Ok(Err(pending_ts));
		})
		.custom_instrument(tracing::info_span!("actor_allocate_tx"))
		.await?;

	let dt = start_instant.elapsed().as_secs_f64();
	metrics::ACTOR_ALLOCATE_DURATION
		.with_label_values(&[&res.is_ok().to_string()])
		.observe(dt);

	Ok(res)
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct UpdateFdbInput {
	pub actor_id: util::Id,
	pub client_id: Uuid,
	pub state: protocol::ActorState,
}

#[activity(UpdateFdb)]
pub async fn update_fdb(ctx: &ActivityCtx, input: &UpdateFdbInput) -> GlobalResult<()> {
	use protocol::ActorState::*;

	match &input.state {
		Starting | Running { .. } | Stopping => {}
		Stopped | Lost | Exited { .. } => {
			ctx.fdb()
				.await?
				.run(|tx, _mc| async move {
					// Was inserted when the actor was allocated. This is cleared when the state changes as
					// well as when the actor is destroyed to ensure consistency during rescheduling and
					// forced deletion.
					let actor_key = keys::client::Actor2Key::new(input.client_id, input.actor_id);
					tx.clear(&keys::subspace().pack(&actor_key));

					Ok(())
				})
				.custom_instrument(tracing::info_span!("actor_clear_tx"))
				.await?;
		}
	}

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct UpdateImageInput {
	pub image_id: Uuid,
}

#[activity(UpdateImage)]
pub async fn update_image(ctx: &ActivityCtx, input: &UpdateImageInput) -> GlobalResult<()> {
	let pool = ctx.sqlite().await?;

	sql_execute!(
		[ctx, pool]
		"
		UPDATE state
		SET image_id = ?
		",
		input.image_id,
	)
	.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct SetStartedInput {
	pub actor_id: util::Id,
	pub generation: u32,
}

#[derive(Serialize)]
pub(crate) struct ActorRunnerClickhouseRow {
	pub actor_id: String,
	pub generation: u32,
	pub runner_id: Uuid,
	pub started_at: i64,
	pub finished_at: i64,
}

#[activity(SetStarted)]
pub async fn set_started(ctx: &ActivityCtx, input: &SetStartedInput) -> GlobalResult<()> {
	let pool = ctx.sqlite().await?;
	let start_ts = util::timestamp::now();

	let (create_ts, old_start_ts, runner_id, old_runner_id) = sql_fetch_one!(
		[ctx, (i64, Option<i64>, Uuid, Option<Uuid>), &pool]
		"
		SELECT create_ts, start_ts, runner_id, old_runner_id
		FROM state
		",
		start_ts,
	)
	.await?;

	sql_execute!(
		[ctx, &pool]
		"
		UPDATE state SET start_ts = ?1
		",
		start_ts,
	)
	.await?;

	let inserter = ctx.clickhouse_inserter().await?;

	// Set old alloc as finished
	if let (Some(old_start_ts), Some(old_runner_id)) = (old_start_ts, old_runner_id) {
		inserter.insert(
			"db_pegboard_runner",
			"actor_runners",
			ActorRunnerClickhouseRow {
				actor_id: input.actor_id.to_string(),
				generation: input.generation,
				runner_id: old_runner_id,
				started_at: old_start_ts * 1_000_000, // Convert ms to ns for ClickHouse DateTime64(9)
				finished_at: start_ts * 1_000_000,    // Convert ms to ns for ClickHouse DateTime64(9)
			},
		)?;
	}

	// Insert new alloc
	inserter.insert(
		"db_pegboard_runner",
		"actor_runners",
		ActorRunnerClickhouseRow {
			actor_id: input.actor_id.to_string(),
			generation: input.generation,
			runner_id,
			started_at: start_ts * 1_000_000, // Convert ms to ns for ClickHouse DateTime64(9)
			finished_at: 0,
		},
	)?;

	// Add start metric for first start
	if old_start_ts.is_none() {
		let dt = (start_ts - create_ts) as f64 / 1000.0;
		metrics::ACTOR_START_DURATION
			.with_label_values(&[])
			.observe(dt);
	}

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct SetConnectableInput {
	pub connectable: bool,
}

#[activity(SetConnectable)]
pub async fn set_connectable(ctx: &ActivityCtx, input: &SetConnectableInput) -> GlobalResult<bool> {
	let pool = ctx.sqlite().await?;

	let res = sql_execute!(
		[ctx, pool]
		"
		UPDATE state
		SET connectable_ts = ?
		WHERE
			CASE WHEN ?
			THEN connectable_ts IS NULL
			ELSE connectable_ts IS NOT NULL
			END
		",
		input.connectable.then(util::timestamp::now),
		input.connectable,
	)
	.await?;

	Ok(res.rows_affected() > 0)
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct InsertPortsInput {
	pub ports: util::serde::HashableMap<String, protocol::ProxiedPort>,
}

#[activity(InsertPorts)]
pub async fn insert_ports(ctx: &ActivityCtx, input: &InsertPortsInput) -> GlobalResult<()> {
	let pool = ctx.sqlite().await?;
	let mut conn = pool.conn().await?;
	let mut tx = conn.begin().await?;

	for (port_name, port) in &input.ports {
		sql_execute!(
			[ctx, @tx &mut tx]
			"
			INSERT INTO ports_proxied (
				port_name,
				source,
				ip
			)
			VALUES (?, ?, ?)
			",
			port_name,
			port.source as i64,
			&port.lan_hostname,
		)
		.await?;
	}

	tx.commit().await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct InsertPortsFdbInput {
	pub actor_id: util::Id,
	pub ports: util::serde::HashableMap<String, protocol::ProxiedPort>,
}

#[activity(InsertPortsFdb)]
pub async fn insert_ports_fdb(ctx: &ActivityCtx, input: &InsertPortsFdbInput) -> GlobalResult<()> {
	let pool = &ctx.sqlite().await?;

	let ((create_ts,), ingress_ports) = tokio::try_join!(
		sql_fetch_one!(
			[ctx, (i64,), pool]
			"
			SELECT create_ts
			FROM state 
			",
		),
		sql_fetch_all!(
			[ctx, (String, i64, i64), pool]
			"
			SELECT port_name, ingress_port_number, protocol
			FROM ports_ingress 
			",
		),
	)?;

	let proxied_ports = input
		.ports
		.iter()
		// Match to ingress ports for GG
		.filter_map(|(port_name, port)| {
			if let Some((_, ingress_port_number, protocol)) = ingress_ports
				.iter()
				.find(|(ingress_port_name, _, _)| port_name == ingress_port_name)
			{
				Some((port_name, port, ingress_port_number, protocol))
			} else {
				None
			}
		})
		.map(|(port_name, port, ingress_port_number, protocol)| {
			let protocol = unwrap!(GameGuardProtocol::from_repr((*protocol).try_into()?));

			Ok(keys::actor2::ProxiedPort {
				port_name: port_name.clone(),
				create_ts,
				lan_hostname: port.lan_hostname.clone(),
				source: port.source,
				ingress_port_number: (*ingress_port_number).try_into()?,
				protocol,
			})
		})
		.collect::<GlobalResult<Vec<_>>>()?;

	// Write proxied ingress ports to fdb index
	ctx.fdb()
		.await?
		.run(|tx, _mc| {
			let proxied_ports = proxied_ports.clone();
			async move {
				let proxied_ports_key = keys::actor2::ProxiedPortsKey::new(input.actor_id);

				tx.set(
					&keys::subspace().pack(&proxied_ports_key),
					&proxied_ports_key
						.serialize(proxied_ports)
						.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
				);

				Ok(())
			}
		})
		.custom_instrument(tracing::info_span!("actor_insert_proxied_ports_tx"))
		.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct CompareRetryInput {
	last_retry_ts: i64,
}

#[activity(CompareRetry)]
async fn compare_retry(ctx: &ActivityCtx, input: &CompareRetryInput) -> GlobalResult<(i64, bool)> {
	let now = util::timestamp::now();

	// If the last retry ts is more than RETRY_RESET_DURATION_MS, reset retry count
	Ok((now, input.last_retry_ts < now - RETRY_RESET_DURATION_MS))
}

/// Returns None if a destroy signal was received while pending for allocation.
pub async fn spawn_actor(
	ctx: &mut WorkflowCtx,
	input: &Input,
	actor_setup: &setup::ActorSetupCtx,
	generation: u32,
) -> GlobalResult<Option<AllocateActorOutput>> {
	// Attempt allocation
	let allocate_res = ctx
		.activity(AllocateActorInput {
			actor_id: input.actor_id,
			generation,
			image_id: actor_setup.image_id,
			build_allocation_type: actor_setup.meta.build_allocation_type,
			build_allocation_total_slots: actor_setup.meta.build_allocation_total_slots,
			resources: actor_setup.resources.clone(),
		})
		.await?;

	let allocate_res = match allocate_res {
		Ok(x) => x,
		Err(pending_allocation_ts) => {
			tracing::warn!(
				actor_id=?input.actor_id,
				"failed to allocate (no availability), waiting for allocation",
			);

			ctx.activity(SetPendingAllocationInput {
				pending_allocation_ts,
			})
			.await?;

			// If allocation fails, the allocate txn already inserted this actor into the queue. Now we wait for
			// an `Allocate` signal
			match ctx.listen::<PendingAllocation>().await? {
				PendingAllocation::Allocate(sig) => AllocateActorOutput {
					runner_id: sig.runner_id,
					new_runner: sig.new_runner,
					client_id: sig.client_id,
					client_workflow_id: sig.client_workflow_id,
				},
				// We ignore the signal's override_kill_timeout_ms because the actor isn't allocated
				PendingAllocation::Destroy(_sig) => {
					tracing::debug!("destroying before actor allocated");

					let cleared = ctx
						.activity(ClearPendingAllocationInput {
							actor_id: input.actor_id,
							pending_allocation_ts,
						})
						.await?;

					// If this actor was no longer present in the queue it means it was allocated. We must now
					// wait for the allocated signal to prevent a race condition.
					if !cleared {
						let sig = ctx.listen::<Allocate>().await?;

						ctx.activity(UpdateClientAndRunnerInput {
							client_id: sig.client_id,
							client_workflow_id: sig.client_workflow_id,
							runner_id: sig.runner_id,
						})
						.await?;
					}

					return Ok(None);
				}
			}
		}
	};

	let (_, artifacts_res, ports_res) = ctx
		.join((
			activity(UpdateClientAndRunnerInput {
				client_id: allocate_res.client_id,
				client_workflow_id: allocate_res.client_workflow_id,
				runner_id: allocate_res.runner_id,
			}),
			// NOTE: We resolve the artifacts here instead of in setup::setup because we don't know how
			// long it will be after setup until an actor is allocated so the presigned artifact url might
			// expire.
			activity(ResolveArtifactsInput {
				build_upload_id: actor_setup.meta.build_upload_id,
				build_file_name: actor_setup.meta.build_file_name.clone(),
				dc_build_delivery_method: actor_setup.meta.dc_build_delivery_method,
			}),
			activity(FetchPortsInput {
				actor_id: input.actor_id,
				endpoint_type: input.endpoint_type,
			}),
		))
		.await?;

	let cluster_id = ctx.config().server()?.rivet.edge()?.cluster_id;

	let image = protocol::Image {
		id: actor_setup.image_id,
		artifact_url_stub: artifacts_res.artifact_url_stub.clone(),
		fallback_artifact_url: Some(artifacts_res.fallback_artifact_url.clone()),
		artifact_size: artifacts_res.artifact_size,
		kind: match actor_setup.meta.build_kind {
			BuildKind::DockerImage => protocol::ImageKind::DockerImage,
			BuildKind::OciBundle => protocol::ImageKind::OciBundle,
			BuildKind::JavaScript => bail!("actors do not support js builds"),
		},
		compression: actor_setup.meta.build_compression.into(),
		allocation_type: match actor_setup.meta.build_allocation_type {
			BuildAllocationType::None => bail!("actors do not support old builds"),
			BuildAllocationType::Single => protocol::ImageAllocationType::Single,
			BuildAllocationType::Multi => protocol::ImageAllocationType::Multi,
		},
	};
	let ports = ports_res
		.ports
		.iter()
		.map(|port| match port.port.routing {
			Routing::GameGuard { protocol } => (
				crate::util::pegboard_normalize_port_name(&port.name),
				protocol::Port {
					target: port.port_number,
					protocol: match protocol {
						GameGuardProtocol::Http
						| GameGuardProtocol::Https
						| GameGuardProtocol::Tcp
						| GameGuardProtocol::TcpTls => protocol::TransportProtocol::Tcp,
						GameGuardProtocol::Udp => protocol::TransportProtocol::Udp,
					},
					routing: protocol::PortRouting::GameGuard,
				},
			),
			Routing::Host { protocol } => (
				crate::util::pegboard_normalize_port_name(&port.name),
				protocol::Port {
					target: port.port_number,
					protocol: match protocol {
						HostProtocol::Tcp => protocol::TransportProtocol::Tcp,
						HostProtocol::Udp => protocol::TransportProtocol::Udp,
					},
					routing: protocol::PortRouting::Host,
				},
			),
		})
		.collect::<util::serde::HashableMap<_, _>>();
	let network_mode = match input.network_mode {
		NetworkMode::Bridge => protocol::NetworkMode::Bridge,
		NetworkMode::Host => protocol::NetworkMode::Host,
	};

	ctx.signal(protocol::Command::StartActor {
		actor_id: input.actor_id,
		generation,
		config: Box::new(protocol::ActorConfig {
			runner: if allocate_res.new_runner {
				Some(protocol::ActorRunner::New {
					runner_id: allocate_res.runner_id,
					config: protocol::RunnerConfig {
						image: image.clone(),
						root_user_enabled: input.root_user_enabled,
						resources: actor_setup.resources.clone(),
						env: input.environment.clone(),
						ports: ports.clone(),
						network_mode,
					},
				})
			} else {
				Some(protocol::ActorRunner::Existing {
					runner_id: allocate_res.runner_id,
				})
			},
			env: input.environment.clone(),
			metadata: util::serde::Raw::new(&protocol::ActorMetadata {
				actor: protocol::ActorMetadataActor {
					actor_id: input.actor_id,
					tags: input.tags.clone(),
					create_ts: ctx.ts(),
				},
				network: Some(protocol::ActorMetadataNetwork {
					ports: ports_res
						.ports
						.into_iter()
						.map(|port| (port.name, port.port))
						.collect(),
				}),
				project: protocol::ActorMetadataProject {
					project_id: actor_setup.meta.project_id,
					slug: actor_setup.meta.project_slug.clone(),
				},
				environment: protocol::ActorMetadataEnvironment {
					env_id: input.env_id,
					slug: actor_setup.meta.env_slug.clone(),
				},
				datacenter: protocol::ActorMetadataDatacenter {
					name_id: actor_setup.meta.dc_name_id.clone(),
					display_name: actor_setup.meta.dc_display_name.clone(),
				},
				cluster: protocol::ActorMetadataCluster { cluster_id },
				build: protocol::ActorMetadataBuild {
					build_id: actor_setup.image_id,
				},
			})?,

			// Deprecated
			image,
			root_user_enabled: input.root_user_enabled,
			resources: actor_setup.resources.clone(),
			ports,
			network_mode,
		}),
	})
	.to_workflow_id(allocate_res.client_workflow_id)
	.send()
	.await?;

	Ok(Some(allocate_res))
}

/// Returns true if the actor should be destroyed.
pub async fn reschedule_actor(
	ctx: &mut WorkflowCtx,
	input: &Input,
	state: &mut State,
	image_id: Uuid,
) -> GlobalResult<bool> {
	tracing::debug!(actor_id=?input.actor_id, "rescheduling actor");

	let res = ctx
		.activity(ClearPortsAndResourcesInput {
			actor_id: input.actor_id,
			image_id,
			runner_id: state.runner_id,
			client_id: state.client_id,
			client_workflow_id: state.client_workflow_id,
		})
		.await?;

	// `destroy_runner` is true when this was the last actor running on that runner, meaning we have to
	// destroy it.
	if res.destroy_runner {
		ctx.signal(protocol::Command::SignalRunner {
			runner_id: state.runner_id,
			signal: Signal::SIGKILL as i32,
		})
		.to_workflow_id(state.client_workflow_id)
		.send()
		.await?;
	}

	let actor_setup = setup::setup(ctx, &input, setup::SetupCtx::Reschedule { image_id }).await?;

	let next_generation = state.generation + 1;

	// Waits for the actor to be ready (or destroyed) and automatically retries if failed to allocate.
	let res = ctx
		.loope(state.reschedule_state.clone(), |ctx, state| {
			let input = input.clone();
			let actor_setup = actor_setup.clone();

			async move {
				// Determine next backoff sleep duration
				let mut backoff =
					util::Backoff::new_at(8, None, BASE_RETRY_TIMEOUT_MS, 500, state.retry_count);

				let (now, reset) = ctx
					.v(2)
					.activity(CompareRetryInput {
						last_retry_ts: state.last_retry_ts,
					})
					.await?;

				state.retry_count = if reset { 0 } else { state.retry_count + 1 };
				state.last_retry_ts = now;

				// Don't sleep for first retry
				if state.retry_count > 0 {
					let next = backoff.step().expect("should not have max retry");

					// Sleep for backoff or destroy early
					if let Some(_sig) = ctx
						.listen_with_timeout::<Destroy>(Instant::from(next) - Instant::now())
						.await?
					{
						tracing::debug!("destroying before actor start");

						return Ok(Loop::Break(None));
					}
				}

				if let Some(res) = spawn_actor(ctx, &input, &actor_setup, next_generation).await? {
					Ok(Loop::Break(Some((state.clone(), res))))
				} else {
					// Destroyed early
					Ok(Loop::Break(None))
				}
			}
			.boxed()
		})
		.await?;

	// Update loop state
	if let Some((reschedule_state, res)) = res {
		state.generation = next_generation;
		state.runner_id = res.runner_id;
		state.client_id = res.client_id;
		state.client_workflow_id = res.client_workflow_id;

		// Save reschedule state in global state
		state.reschedule_state = reschedule_state;

		// Reset gc timeout once allocated
		state.gc_timeout_ts = Some(util::timestamp::now() + ACTOR_START_THRESHOLD_MS);

		Ok(false)
	} else {
		Ok(true)
	}
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct SetPendingAllocationInput {
	pending_allocation_ts: i64,
}

#[activity(SetPendingAllocation)]
pub async fn set_pending_allocation(
	ctx: &ActivityCtx,
	input: &SetPendingAllocationInput,
) -> GlobalResult<()> {
	let pool = ctx.sqlite().await?;

	sql_execute!(
		[ctx, pool]
		"
		UPDATE state
		SET pending_allocation_ts = ?
		",
		input.pending_allocation_ts,
	)
	.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct ClearPendingAllocationInput {
	actor_id: util::Id,
	pending_allocation_ts: i64,
}

#[activity(ClearPendingAllocation)]
pub async fn clear_pending_allocation(
	ctx: &ActivityCtx,
	input: &ClearPendingAllocationInput,
) -> GlobalResult<bool> {
	// Clear self from alloc queue
	let cleared = ctx
		.fdb()
		.await?
		.run(|tx, _mc| async move {
			let pending_alloc_key = keys::subspace().pack(&keys::datacenter::PendingActorKey::new(
				input.pending_allocation_ts,
				input.actor_id,
			));

			let exists = tx.get(&pending_alloc_key, SERIALIZABLE).await?.is_some();

			tx.clear(&pending_alloc_key);

			Ok(exists)
		})
		.await?;

	Ok(cleared)
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct ClearPortsAndResourcesInput {
	actor_id: util::Id,
	image_id: Uuid,
	runner_id: Uuid,
	client_id: Uuid,
	client_workflow_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct ClearPortsAndResourcesOutput {
	destroy_runner: bool,
}

#[activity(ClearPortsAndResources)]
async fn clear_ports_and_resources(
	ctx: &ActivityCtx,
	input: &ClearPortsAndResourcesInput,
) -> GlobalResult<ClearPortsAndResourcesOutput> {
	let pool = &ctx.sqlite().await?;

	let (
		build_res,
		ingress_ports,
		(selected_resources_cpu_millicores, selected_resources_memory_mib),
		_,
	) = tokio::try_join!(
		ctx.op(build::ops::get::Input {
			build_ids: vec![input.image_id],
		}),
		sql_fetch_all!(
			[ctx, (i64, i64), pool]
			"
			SELECT protocol, ingress_port_number
			FROM ports_ingress
			",
		),
		sql_fetch_one!(
			[ctx, (Option<i64>, Option<i64>), pool]
			"
			SELECT selected_resources_cpu_millicores, selected_resources_memory_mib
			FROM state
			",
		),
		// Idempotent
		sql_execute!(
			[ctx, pool]
			"
			DELETE FROM ports_proxied
			",
		),
	)?;
	let build = unwrap_with!(build_res.builds.first(), BUILD_NOT_FOUND);

	let destroy_runner = ctx
		.fdb()
		.await?
		.run(|tx, _mc| {
			let ingress_ports = ingress_ports.clone();
			async move {
				destroy::clear_ports_and_resources(
					input.actor_id,
					input.image_id,
					Some(build.allocation_type),
					ingress_ports,
					Some(input.runner_id),
					Some(input.client_id),
					Some(input.client_workflow_id),
					selected_resources_memory_mib,
					selected_resources_cpu_millicores,
					&tx,
				)
				.await
			}
		})
		.custom_instrument(tracing::info_span!("actor_clear_ports_and_resources_tx"))
		.await?;

	Ok(ClearPortsAndResourcesOutput { destroy_runner })
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct SetFinishedInput {}

#[activity(SetFinished)]
pub async fn set_finished(ctx: &ActivityCtx, input: &SetFinishedInput) -> GlobalResult<()> {
	let pool = ctx.sqlite().await?;

	sql_execute!(
		[ctx, pool]
		"
		UPDATE state
		SET finish_ts = ?
		",
		util::timestamp::now(),
	)
	.await?;

	Ok(())
}
