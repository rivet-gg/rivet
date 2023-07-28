use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};
use util_db::assert_ident_snake;

#[worker(name = "db-create")]
async fn worker(ctx: OperationContext<db::msg::create::Message>) -> GlobalResult<()> {
	let crdb = ctx.crdb("db-db").await?;
	let pg_data = ctx.crdb("db-db-data").await?;

	let database_id = internal_unwrap!(ctx.database_id).as_uuid();
	let owner_team_id = internal_unwrap!(ctx.owner_team_id).as_uuid();

	// Serialize default schema
	let default_schema = backend::db::Schema {
		collections: Vec::new(),
	};
	let mut schema_buf = Vec::with_capacity(default_schema.encoded_len());
	default_schema.encode(&mut schema_buf)?;

	// Create database
	let schema_name = util_db::schema_name(database_id);
	sqlx::query(&format!(
		r#"CREATE SCHEMA "{db}""#,
		db = assert_ident_snake(&schema_name)?
	))
	.execute(&pg_data)
	.await?;

	// Save database
	sqlx::query(indoc!(
		"
		INSERT INTO databases (database_id, owner_team_id, name_id, create_ts, schema)
		VALUES ($1, $2, $3, $4, $5)
		"
	))
	.bind(database_id)
	.bind(owner_team_id)
	.bind(&ctx.name_id)
	.bind(ctx.ts())
	.bind(schema_buf)
	.execute(&crdb)
	.await?;

	msg!([ctx] db::msg::create_complete(database_id) {
		database_id: Some(database_id.into()),
	})
	.await?;

	Ok(())
}
