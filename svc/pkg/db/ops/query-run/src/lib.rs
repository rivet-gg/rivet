use futures_util::{StreamExt, TryStreamExt};
use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
use std::{collections::HashMap, convert::TryInto};
use util_db::{ais, entry_id::EntryId};

#[derive(scylla::macros::FromRow)]
struct GetRow {
	entry_id: Uuid,
	value: Vec<u8>,
}

impl TryInto<db::query_run::response::Entry> for GetRow {
	type Error = GlobalError;

	fn try_into(self) -> GlobalResult<db::query_run::response::Entry> {
		// Convert value to JSON string
		let value = bson::from_slice::<serde_json::Value>(self.value.as_slice())?;
		let json = serde_json::to_string(&value)?;

		Ok(db::query_run::response::Entry {
			entry_id: Some(self.entry_id.into()),
			value: json,
		})
	}
}

rivet_pools::cass_prepared_statement!(get_entry => indoc!(
	"
	SELECT entry_id, value
	FROM kv
	WHERE database_id = ? AND collection = ? AND entry_id IN ?
	"
));

rivet_pools::cass_prepared_statement!(insert_entry => indoc!(
	"
	INSERT INTO kv (database_id, collection, entry_id, value)
	VALUES (?, ?, ?, ?)
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
	let (schema_buf,) = sqlx::query_as::<_, (Vec<u8>,)>(indoc!(
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
	let res = run_query(&cass_data, database_id, &schema, query).await?;

	Ok(res)
}

async fn run_query(
	cass_data: &CassPool,
	database_id: Uuid,
	schema: &backend::db::Schema,
	query: &backend::db::Query,
) -> GlobalResult<db::query_run::Response> {
	match internal_unwrap!(query.kind) {
		backend::db::query::Kind::Get(get) => {
			let collection = get_collection(schema, &get.collection)?;

			let entry_ids = get
				.entry_ids
				.iter()
				.map(common::Uuid::as_uuid)
				.collect::<Vec<_>>();

			let entries = cass_data
				.execute(
					get_entry::prepare(&cass_data).await?,
					(database_id, &get.collection, &entry_ids),
				)
				.await?
				.rows_typed::<GetRow>()?
				.collect::<Result<Vec<_>, _>>()?
				.into_iter()
				.map(TryInto::try_into)
				.collect::<GlobalResult<Vec<_>>>()?;

			Ok(db::query_run::Response {
				entries,
				..Default::default()
			})
		}
		backend::db::query::Kind::Insert(insert) => {
			let collection = get_collection(schema, &insert.collection)?;

			// TODO: Reduce clones
			// TODO: This has a problem if one entry fails to insert but the rest succeed
			// Insert entries
			let entry_ids = futures_util::stream::iter(insert.entries.clone())
				.map({
					let cass_data = cass_data.clone();
					let collection = insert.collection.clone();
					move |entry| {
						let cass_data = cass_data.clone();
						let collection = collection.clone();

						let value = serde_json::from_str::<serde_json::Value>(&entry.value)?;
						let value_bson = bson::to_vec(&value)?;

						let entry_id = Uuid::new_v4();

						GlobalResult::Ok(async move {
							cass_data
								.execute(
									insert_entry::prepare(&cass_data).await?,
									(database_id, collection, entry_id, value_bson),
								)
								.await?;
							GlobalResult::Ok(common::Uuid::from(entry_id))
						})
					}
				})
				.try_buffer_unordered(16)
				.try_collect::<Vec<_>>()
				.await?;

			Ok(db::query_run::Response {
				entry_ids,
				..Default::default()
			})
		}
		backend::db::query::Kind::Update(update) => {
			todo!()
		}
		backend::db::query::Kind::Delete(delete) => {
			todo!()
		}
		backend::db::query::Kind::Query(query) => {
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

// fn push_filters(
// 	collection: &backend::db::Collection,
// 	query: &mut sqlx::QueryBuilder<sqlx::Postgres>,
// 	filters: &[backend::db::Filter],
// ) -> GlobalResult<()> {
// 	for (i, filter) in filters.iter().enumerate() {
// 		let field = SqlField::from_user_defined_or_internal(collection, &filter.field)?;
// 		match internal_unwrap!(filter.kind) {
// 			backend::db::filter::Kind::Equal(value) => {
// 				field.push_column_name(query)?;
// 				query.push(" = ");
// 				field.bind_value(query, &value)?;
// 			}
// 		}
// 		if i != filters.len() - 1 {
// 			query.push(" AND ");
// 		}
// 	}
// 	Ok(())
// }
