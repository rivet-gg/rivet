use chirp_worker::prelude::*;
use proto::backend::pkg::*;

lazy_static::lazy_static! {
	static ref REDIS_SCRIPT: redis::Script = redis::Script::new(include_str!("../../redis-scripts/datacenter_closed_set.lua"));
}

#[derive(Debug, sqlx::FromRow)]
struct LobbyRow {
	lobby_id: Uuid,
	namespace_id: Uuid,
	lobby_group_id: Uuid,
	max_players_normal: i64,
	max_players_party: i64,
}

#[worker(name = "mm-datacenter-closed-set")]
async fn worker(
	ctx: &OperationContext<mm::msg::datacenter_closed_set::Message>,
) -> GlobalResult<()> {
	let datacenter_id = unwrap_ref!(ctx.datacenter_id).as_uuid();

	let lobby_rows = sql_fetch_all!(
		[ctx, LobbyRow]
		"
		UPDATE db_mm_state.lobbies
		SET is_closed = $2
		WHERE region_id = $1
		RETURNING lobby_id, namespace_id, lobby_group_id, max_players_normal, max_players_party
		",
		datacenter_id,
		ctx.is_closed,
	)
	.await?;

	// Update matchmaking index
	if ctx.is_closed {
		let mut pipe = redis::pipe();
		pipe.atomic();

		pipe.set(util_mm::key::datacenter_is_closed(datacenter_id), true);

		for lobby in lobby_rows {
			pipe.zrem(
				util_mm::key::lobby_available_spots(
					lobby.namespace_id,
					datacenter_id,
					lobby.lobby_group_id,
					util_mm::JoinKind::Normal,
				),
				lobby.lobby_id.to_string(),
			)
			.zrem(
				util_mm::key::lobby_available_spots(
					lobby.namespace_id,
					datacenter_id,
					lobby.lobby_group_id,
					util_mm::JoinKind::Party,
				),
				lobby.lobby_id.to_string(),
			);
		}

		pipe.query_async(&mut ctx.redis_mm().await?).await?;
	} else {
		let mut script = REDIS_SCRIPT.prepare_invoke();

		script
			.key(util_mm::key::datacenter_is_closed(datacenter_id))
			.arg(lobby_rows.len());

		for lobby in lobby_rows {
			script
				.key(util_mm::key::lobby_config(lobby.lobby_id))
				.key(util_mm::key::lobby_player_ids(lobby.lobby_id))
				.key(util_mm::key::lobby_available_spots(
					lobby.namespace_id,
					datacenter_id,
					lobby.lobby_group_id,
					util_mm::JoinKind::Normal,
				))
				.key(util_mm::key::lobby_available_spots(
					lobby.namespace_id,
					datacenter_id,
					lobby.lobby_group_id,
					util_mm::JoinKind::Party,
				))
				.arg(lobby.lobby_id.to_string())
				.arg(lobby.max_players_normal)
				.arg(lobby.max_players_party);
		}

		script.invoke_async(&mut ctx.redis_mm().await?).await?;
	}

	Ok(())
}
