use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
use sqlx::Row;
use std::collections::HashMap;
use util_db::ais;

use crate::sql_field::SqlField;

mod sql_field;

#[operation(name = "db-query-run")]
pub async fn handle(
	ctx: OperationContext<db::query_run::Request>,
) -> GlobalResult<db::query_run::Response> {
	let crdb = ctx.crdb("db-db").await?;
	let pg_data = ctx.postgres("db-db-data").await?;

	let database_id = internal_unwrap!(ctx.database_id).as_uuid();
	let query = internal_unwrap!(ctx.query);

	// Read database
	let (database_id_short, schema_buf) = sqlx::query_as::<_, (String, Vec<u8>)>(indoc!(
		"
		SELECT database_id_short, schema
		FROM databases
		WHERE database_id = $1
		"
	))
	.bind(database_id)
	.fetch_one(&crdb)
	.await?;

	tracing::info!(?database_id_short);

	// Parse schema
	let schema = backend::db::Schema::decode(schema_buf.as_slice())?;

	// Run query
	let res = run_query(&pg_data, &database_id_short, &schema, query).await?;

	Ok(res)
}

async fn run_query(
	pg_data: &PostgresPool,
	database_id_short: &str,
	schema: &backend::db::Schema,
	query: &backend::db::Query,
) -> GlobalResult<db::query_run::Response> {
	let schema_name = util_db::schema_name(database_id_short);

	// TODO: Do bulk inserts with https://github.com/launchbadge/sqlx/blob/main/FAQ.md#how-can-i-bind-an-array-to-a-values-clause-how-can-i-do-bulk-inserts

	match internal_unwrap!(query.kind) {
		backend::db::query::Kind::Fetch(fetch) => {
			let collection = get_collection(schema, &fetch.collection)?;

			let mut fields = vec![SqlField::Id];
			fields.extend(
				collection
					.fields
					.iter()
					.map(SqlField::from_user_defined)
					.collect::<GlobalResult<Vec<_>>>()?,
			);

			let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT ");

			// Specify columns
			for (i, field) in fields.iter().enumerate() {
				field.push_column_name(&mut query)?;
				if i != fields.len() - 1 {
					query.push(", ");
				}
			}
			query.push(" ");

			// Specify table
			let table = util_db::table_name(&collection.name_id);
			query.push(format!(
				r#"FROM "{schema}"."{table}" "#,
				schema = ais(&schema_name)?,
				table = ais(&table)?
			));

			// Specify filters
			query.push("WHERE ");
			for (i, filter) in fetch.filters.iter().enumerate() {
				let field = SqlField::from_user_defined_or_internal(collection, &filter.field)?;
				match internal_unwrap!(filter.kind) {
					backend::db::filter::Kind::Equal(value) => {
						field.push_column_name(&mut query)?;
						query.push(" = ");
						field.bind_value(&mut query, value)?;
					}
				}

				if i != fetch.filters.len() - 1 {
					query.push(" AND ");
				}
			}
			query.push(" ");

			// Run query
			tracing::info!(sql = ?query.sql(), "running get");
			let rows = query.build().fetch_all(pg_data).await?;

			let mut fetched_entries = Vec::new();
			for row in rows {
				internal_assert_eq!(fields.len(), row.len());

				// Decode response
				let mut entry = HashMap::new();
				for (i, field) in fields.iter().enumerate() {
					let value = field.get_column(&row, i)?;
					entry.insert(field.field_name_id().to_string(), value);
				}

				fetched_entries.push(db::query_run::response::FetchEntry { entry });
			}

			Ok(db::query_run::Response {
				fetched_entries,
				inserted_entries: Vec::new(),
			})
		}
		backend::db::query::Kind::Insert(insert) => {
			let collection = get_collection(schema, &insert.collection)?;

			let entry_fields = insert
				.entry
				.iter()
				.map(|(field, value)| {
					Ok((
						SqlField::from_user_defined_name_id(collection, field)?,
						value,
					))
				})
				.collect::<GlobalResult<Vec<_>>>()?;

			let table = util_db::table_name(&collection.name_id);
			let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new(format!(
				r#"INSERT INTO "{schema}"."{table}" ("#,
				schema = ais(&schema_name)?,
				table = ais(&table)?,
			));

			// Specify columns
			for (i, (field, _)) in entry_fields.iter().enumerate() {
				field.push_column_name(&mut query)?;
				if i != entry_fields.len() - 1 {
					query.push(", ");
				}
			}

			// Bind values
			query.push(") VALUES (");
			for (i, (field, value)) in entry_fields.iter().enumerate() {
				field.bind_value(&mut query, value)?;
				if i != entry_fields.len() - 1 {
					query.push(", ");
				}
			}
			query.push(") RETURNING (id)");

			// Run query
			tracing::info!(sql = ?query.sql(), "running insert");
			let inserted_entries = query
				.build_query_as::<(i64,)>()
				.fetch_all(pg_data)
				.await?
				.into_iter()
				.map(|x| util_db::encode_id(x.0))
				.collect::<GlobalResult<Vec<_>>>()?;

			// TODO: Convert these

			Ok(db::query_run::Response {
				fetched_entries: Vec::new(),
				inserted_entries,
			})
		}
		backend::db::query::Kind::Update(update) => {
			todo!()
		}
		backend::db::query::Kind::Delete(delete) => {
			todo!()
		}
	}
}

/// Returns a collection from a schema.
fn get_collection<'a>(
	schema: &'a backend::db::Schema,
	collection: &str,
) -> GlobalResult<&'a backend::db::Collection> {
	let x = unwrap_with_owned!(
		schema.collections.iter().find(|x| x.name_id == collection),
		DB_COLLECTION_NOT_FOUND,
		collection = collection
	);
	Ok(x)
}
