use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
use sqlx::Row;
use std::{collections::HashMap, convert::TryInto};
use util_db::{ais, entry_id::EntryId};

use crate::sql_field::SqlField;

mod sql_field;

#[derive(scylla::macros::FromRow)]
struct GetRow {
	key: String,
	value: Vec<u8>,
}

rivet_pools::cass_prepared_statement!(get_entry => indoc!(
	"
	SELECT key, value
	FROM uploads
	WHERE database_id = ? AND collection = ? AND key IN ?
	"
));

#[operation(name = "db-query-run")]
pub async fn handle(
	ctx: OperationContext<db::query_run::Request>,
) -> GlobalResult<db::query_run::Response> {
	let crdb = ctx.crdb("db-db").await?;
	let cass_data = ctx.cass("db-db-data").await?;

	let database_id = internal_unwrap!(ctx.database_id).as_uuid();
	let query = internal_unwrap!(ctx.query);

	// Read database
	let (schema_buf,) = sqlx::query_as::<_, (String, Vec<u8>)>(indoc!(
		"
		SELECT schema
		FROM databases
		WHERE database_id = $1
		"
	))
	.bind(database_id)
	.fetch_one(&crdb)
	.await?;

	// Parse schema
	let schema = backend::db::Schema::decode(schema_buf.as_slice())?;

	// Run query
	let res = run_query(&cass_data, &schema, query).await?;

	Ok(res)
}

async fn run_query(
	cass_data: &CassPool,
	database_id: &str,
	schema: &backend::db::Schema,
	query: &backend::db::Query,
) -> GlobalResult<db::query_run::Response> {
	// TODO: Do bulk inserts with https://github.com/launchbadge/sqlx/blob/main/FAQ.md#how-can-i-bind-an-array-to-a-values-clause-how-can-i-do-bulk-inserts

	match internal_unwrap!(query.kind) {
		backend::db::query::Kind::Get(get) => {
			let collection = get_collection(schema, &get.collection)?;

			cass_data
				.execute(
					get_entry::prepare(&cass_data).await?,
					(database_id, get.collection, entry_ids),
				)
				.await?
				.rows_typed::<GetRow>()?
				.collect::<Result<Vec<_>, _>>()?
				.into_iter()
				.map(Into::into)
				.collect();

			Ok(db::query_run::Response {
				fetched_entries,
				inserted_entries: Vec::new(),
				entries_affected: 0,
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
				.map(|x| EntryId::new(x.0).encode())
				.collect::<GlobalResult<Vec<_>>>()?;
			let entries_affected: u64 = inserted_entries.len().try_into()?;

			Ok(db::query_run::Response {
				fetched_entries: Vec::new(),
				inserted_entries,
				entries_affected,
			})
		}
		backend::db::query::Kind::Update(update) => {
			let collection = get_collection(schema, &update.collection)?;

			let set_fields = update
				.set
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
				r#"UPDATE "{schema}"."{table}" "#,
				schema = ais(&schema_name)?,
				table = ais(&table)?,
			));

			// Set values
			query.push("SET ");
			for (i, (field, value)) in set_fields.iter().enumerate() {
				field.push_column_name(&mut query)?;
				query.push(" = ");
				field.bind_value(&mut query, value)?;
				if i != set_fields.len() - 1 {
					query.push(", ");
				}
			}
			query.push(" ");

			// Specify filters
			query.push("WHERE ");
			push_filters(&collection, &mut query, &update.filters)?;
			query.push(" ");

			// Run query
			tracing::info!(sql = ?query.sql(), "running update");
			let output = query.build().execute(pg_data).await?;
			let entries_affected = output.rows_affected();

			Ok(db::query_run::Response {
				fetched_entries: Vec::new(),
				inserted_entries: Vec::new(),
				entries_affected,
			})
		}
		backend::db::query::Kind::Delete(delete) => {
			let collection = get_collection(schema, &delete.collection)?;

			let mut fields = vec![SqlField::Id];
			fields.extend(
				collection
					.fields
					.iter()
					.map(SqlField::from_user_defined)
					.collect::<GlobalResult<Vec<_>>>()?,
			);

			let table = util_db::table_name(&collection.name_id);
			let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new(format!(
				r#"DELETE FROM "{schema}"."{table}" "#,
				schema = ais(&schema_name)?,
				table = ais(&table)?,
			));

			// Specify filters
			query.push("WHERE ");
			push_filters(&collection, &mut query, &delete.filters)?;
			query.push(" ");

			// Run query
			tracing::info!(sql = ?query.sql(), "running get");
			let output = query.build().execute(pg_data).await?;
			let entries_affected = output.rows_affected();

			Ok(db::query_run::Response {
				fetched_entries: Vec::new(),
				inserted_entries: Vec::new(),
				entries_affected,
			})
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

fn push_filters(
	collection: &backend::db::Collection,
	query: &mut sqlx::QueryBuilder<sqlx::Postgres>,
	filters: &[backend::db::Filter],
) -> GlobalResult<()> {
	for (i, filter) in filters.iter().enumerate() {
		let field = SqlField::from_user_defined_or_internal(collection, &filter.field)?;
		match internal_unwrap!(filter.kind) {
			backend::db::filter::Kind::Equal(value) => {
				field.push_column_name(query)?;
				query.push(" = ");
				field.bind_value(query, &value)?;
			}
		}
		if i != filters.len() - 1 {
			query.push(" AND ");
		}
	}
	Ok(())
}
