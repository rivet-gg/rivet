use proto::backend::pkg::*;
use rivet_operation::prelude::*;

#[operation(name = "db-game-version-publish")]
async fn handle(
	ctx: OperationContext<db::game_version_publish::Request>,
) -> GlobalResult<db::game_version_publish::Response> {
	let crdb = ctx.crdb("db-db").await?;

	let version_id = internal_unwrap!(ctx.version_id).as_uuid();

	let config = internal_unwrap!(ctx.config);
	let schema = internal_unwrap!(config.schema);

	let context = internal_unwrap!(ctx.config_ctx);
	let database_id = internal_unwrap!(context.database_id).as_uuid();

	let mut schema_buf = Vec::with_capacity(schema.encoded_len());
	schema.encode(&mut schema_buf)?;

	sqlx::query(
		"INSERT INTO game_versions (version_id, database_name_id, schema, database_id) VALUES ($1, $2, $3, $4)",
	)
	.bind(version_id)
	.bind(&config.database_name_id)
	.bind(schema_buf)
	.bind(database_id)
	.execute(&crdb)
	.await?;

	Ok(db::game_version_publish::Response {})
}
