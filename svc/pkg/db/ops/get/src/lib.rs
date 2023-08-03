use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

#[derive(sqlx::FromRow)]
struct Database {
	database_id: Uuid,
	owner_team_id: Uuid,
	name_id: String,
	create_ts: i64,
	schema: Vec<u8>,
}

#[operation(name = "db-get")]
pub async fn handle(ctx: OperationContext<db::get::Request>) -> GlobalResult<db::get::Response> {
	let crdb = ctx.crdb("db-db").await?;

	let database_ids = ctx
		.database_ids
		.iter()
		.map(common::Uuid::as_uuid)
		.collect::<Vec<_>>();

	let databases = sqlx::query_as::<_, Database>(indoc!(
		"
		SELECT database_id, owner_team_id, name_id, create_ts, schema
		FROM databases
		WHERE database_id = ANY($1)
		"
	))
	.bind(&database_ids)
	.fetch_all(&crdb)
	.await?
	.into_iter()
	.map(|x| -> GlobalResult<backend::db::Database> {
		let schema = backend::db::Schema::decode(x.schema.as_slice())?;
		Ok(backend::db::Database {
			database_id: Some(x.database_id.into()),
			owner_team_id: Some(x.owner_team_id.into()),
			name_id: x.name_id.clone(),
			create_ts: x.create_ts,
			schema: Some(schema),
		})
	})
	.collect::<GlobalResult<Vec<_>>>()?;

	Ok(db::get::Response { databases })
}
