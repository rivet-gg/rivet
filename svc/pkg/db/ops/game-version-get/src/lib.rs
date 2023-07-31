use std::collections::HashMap;

use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

#[derive(sqlx::FromRow)]
struct GameVersion {
	version_id: Uuid,
	database_name_id: String,
	schema: Vec<u8>,
}

#[operation(name = "db-game-version-get")]
async fn handle(
	ctx: OperationContext<db::game_version_get::Request>,
) -> GlobalResult<db::game_version_get::Response> {
	let crdb = ctx.crdb("db-db").await?;

	let version_ids = ctx
		.version_ids
		.iter()
		.map(common::Uuid::as_uuid)
		.collect::<Vec<_>>();

	let versions = sqlx::query_as::<_, GameVersion>(indoc!(
		"
		SELECT version_id, database_name_id, schema
		FROM game_versions
		WHERE version_id = ANY($1)
		"
	))
	.bind(&version_ids)
	.fetch_all(&crdb)
	.await?
	.into_iter()
	.map(|x| -> GlobalResult<_> {
		let schema = backend::db::Schema::decode(x.schema.as_slice())?;
		Ok(db::game_version_get::response::Version {
			version_id: Some(x.version_id.into()),
			config: Some(backend::db::GameVersionConfig {
				database_name_id: x.database_name_id,
				schema: Some(schema),
			}),
			config_meta: Some(backend::db::GameVersionConfigMeta {}),
		})
	})
	.collect::<GlobalResult<_>>()?;

	Ok(db::game_version_get::Response { versions })
}
