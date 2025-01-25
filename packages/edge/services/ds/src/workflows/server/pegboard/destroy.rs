use std::collections::HashMap;

use chirp_workflow::prelude::*;
use fdb_util::FormalKey;
use foundationdb as fdb;
use nix::sys::signal::Signal;
use pegboard::protocol as pp;
use util::serde::AsHashableExt;

use super::super::{DestroyComplete, DestroyStarted};
use crate::keys;

#[derive(Debug, Serialize, Deserialize)]
pub struct DestroyActor {
	pub actor_id: Uuid,
	pub client_id: Uuid,
	pub kill_timeout_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Input {
	pub server_id: Uuid,
	/// Whether or not to send signals to the pb actor. In the case that the actor was already stopped
	/// or exited, signals are unnecessary.
	pub destroy_actor: Option<DestroyActor>,
}

#[workflow]
pub(crate) async fn ds_server_pegboard_destroy(
	ctx: &mut WorkflowCtx,
	input: &Input,
) -> GlobalResult<()> {
	let ds = ctx.activity(UpdateDbInput {}).await?;

	if let Some(ds) = ds {
		ctx.activity(UpdateFdbIdxInput {
			server_id: input.server_id,
			env_id: ds.env_id,
			tags: ds.tags.as_hashable(),
			create_ts: ds.create_ts,
		})
		.await?;
	}

	ctx.msg(DestroyStarted {})
		.tag("server_id", input.server_id)
		.send()
		.await?;

	if let Some(data) = &input.destroy_actor {
		destroy_actor(
			ctx,
			data.actor_id,
			data.client_id,
			data.kill_timeout_ms,
			false,
		)
		.await?;
	}

	ctx.msg(DestroyComplete {})
		.tag("server_id", input.server_id)
		.send()
		.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct UpdateDbInput {}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct UpdateDbOutput {
	env_id: Uuid,
	tags: sqlx::types::Json<HashMap<String, String>>,
	create_ts: i64,
}

#[activity(UpdateDb)]
async fn update_db(
	ctx: &ActivityCtx,
	input: &UpdateDbInput,
) -> GlobalResult<Option<UpdateDbOutput>> {
	let pool = ctx.sqlite().await?;

	sql_fetch_optional!(
		[ctx, UpdateDbOutput, pool]
		"
		UPDATE state
		SET destroy_ts = ?
		WHERE destroy_ts IS NULL
		RETURNING env_id, tags, create_ts
		",
		ctx.ts(),
	)
	.await
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct UpdateFdbIdxInput {
	server_id: Uuid,
	env_id: Uuid,
	tags: util::serde::HashableMap<String, String>,
	create_ts: i64,
}

#[activity(UpdateFdbIdx)]
async fn update_fdb_idx(ctx: &ActivityCtx, input: &UpdateFdbIdxInput) -> GlobalResult<()> {
	ctx.fdb()
		.await?
		.run(|tx, _mc| async move {
			let server_key =
				keys::env::ServerKey::new(input.env_id, input.create_ts, input.server_id);
			let data = keys::env::ServerKeyData {
				is_destroyed: true,
				tags: input.tags.clone().into_iter().collect(),
			};

			tx.set(
				&keys::subspace().pack(&server_key),
				&server_key
					.serialize(data)
					.map_err(|x| fdb::FdbBindingError::CustomError(x.into()))?,
			);

			Ok(())
		})
		.await?;

	Ok(())
}

pub(crate) async fn destroy_actor(
	ctx: &mut WorkflowCtx,
	actor_id: Uuid,
	client_id: Uuid,
	kill_timeout_ms: i64,
	persist_storage: bool,
) -> GlobalResult<()> {
	if kill_timeout_ms != 0 {
		ctx.signal(pp::Command::SignalActor {
			actor_id,
			signal: Signal::SIGTERM as i32,
			persist_storage,
			ignore_future_state: true,
		})
		.tag("client_id", client_id)
		.send()
		.await?;

		// See `docs/packages/job/JOB_DRAINING_AND_KILL_TIMEOUTS.md`
		ctx.sleep(kill_timeout_ms).await?;
	}

	ctx.signal(pp::Command::SignalActor {
		actor_id,
		signal: Signal::SIGKILL as i32,
		persist_storage,
		ignore_future_state: true,
	})
	.tag("client_id", client_id)
	.send()
	.await?;

	Ok(())
}
