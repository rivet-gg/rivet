use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

#[operation(name = "db-game-version-prepare")]
async fn handle(
	ctx: OperationContext<db::game_version_prepare::Request>,
) -> GlobalResult<db::game_version_prepare::Response> {
	let config = internal_unwrap!(ctx.config);
	let game_id = internal_unwrap!(ctx.game_id).as_uuid();

	// Get team ID
	let game_res = op!([ctx] game_get {
		game_ids: vec![game_id.into()],
	})
	.await?;
	let game = internal_unwrap_owned!(game_res.games.first());
	let developer_team_id = internal_unwrap!(game.developer_team_id).as_uuid();

	// Resolve existing database
	let res = op!([ctx] db_resolve_name_id {
		name_ids: vec![
			db::resolve_name_id::request::Database {
				team_id: Some(developer_team_id.into()),
				name_id: config.database_name_id.clone(),
			}
		]
	})
	.await?;

	// Create database if doesn't exist
	let database_id = if let Some(database) = res.databases.first() {
		// Database already exists
		internal_unwrap!(database.database_id).as_uuid()
	} else {
		// Create database if doesn't exist
		let database_id = Uuid::new_v4();
		msg!([ctx] db::msg::create(database_id) -> db::msg::create_complete {
			database_id: Some(database_id.into()),
			owner_team_id: Some(developer_team_id.into()),
			name_id: config.database_name_id.clone(),
		})
		.await
		.unwrap();

		database_id
	};

	Ok(db::game_version_prepare::Response {
		config_ctx: Some(backend::db::GameVersionConfigCtx {
			database_id: Some(database_id.into()),
		}),
	})
}
