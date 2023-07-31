use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

#[derive(Debug, sqlx::FromRow)]
struct Database {
	owner_team_id: Uuid,
	name_id: String,
	database_id: Uuid,
}

impl From<Database> for db::resolve_name_id::response::Database {
	fn from(value: Database) -> db::resolve_name_id::response::Database {
		db::resolve_name_id::response::Database {
			team_id: Some(value.owner_team_id.into()),
			name_id: value.name_id,
			database_id: Some(value.database_id.into()),
		}
	}
}

#[operation(name = "db-resolve-name-id")]
pub async fn handle(
	ctx: OperationContext<db::resolve_name_id::Request>,
) -> GlobalResult<db::resolve_name_id::Response> {
	let crdb = ctx.crdb("db-db").await?;

	let team_ids = ctx
		.name_ids
		.iter()
		.map(|x| Ok(internal_unwrap!(x.team_id).as_uuid()))
		.collect::<GlobalResult<Vec<Uuid>>>()?;
	let name_ids = ctx
		.name_ids
		.iter()
		.map(|x| x.name_id.as_str())
		.collect::<Vec<_>>();

	let databases = sqlx::query_as::<_, Database>(indoc!(
		"
		SELECT db.owner_team_id, db.name_id, db.database_id
		FROM unnest($1, $2) AS q (team_id, name_id)
		INNER JOIN databases AS db ON db.owner_team_id = q.team_id AND db.name_id = q.name_id
		"
	))
	.bind(&team_ids)
	.bind(&name_ids)
	.fetch_all(&crdb)
	.await?
	.into_iter()
	.map(Into::<db::resolve_name_id::response::Database>::into)
	.collect::<Vec<_>>();

	Ok(db::resolve_name_id::Response { databases })
}
