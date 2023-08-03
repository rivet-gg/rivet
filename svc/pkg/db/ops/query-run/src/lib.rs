use futures_util::{StreamExt, TryStreamExt};
use proto::backend::{
	self,
	db::{
		order_by_schema::Direction as OrderDir, order_by_schema::FieldType as OrderFieldType,
		FieldPath,
	},
	pkg::*,
};
use rivet_operation::prelude::*;
use scylla::frame::value::MaybeUnset;
use serde_json::Value as JsonValue;
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
		let value = bson::from_slice::<JsonValue>(self.value.as_slice())?;
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
	FROM db_db_data.kv
	WHERE database_id = ? AND collection = ? AND entry_id IN ?
	"
));

rivet_pools::cass_prepared_statement!(insert_entry => indoc!(
	"
	INSERT INTO db_db_data.kv (database_id, collection, entry_id, value)
	VALUES (?, ?, ?, ?)
	"
));

rivet_pools::cass_prepared_statement!(insert_entry_index => indoc!(
	"
	INSERT INTO db_db_data.kv_index (database_id, collection, \"index\", group_by, entry_id, entry)
	VALUES (?, ?, ?, ?, ?, ?)
	"
));

rivet_pools::cass_prepared_statement!(insert_entry_index_bigint_asc => indoc!(
	"
	INSERT INTO db_db_data.kv_index_bigint_asc (database_id, collection, \"index\", group_by, entry_id, rank_0, entry)
	VALUES (?, ?, ?, ?, ?, ?, ?)
	"
));

rivet_pools::cass_prepared_statement!(insert_entry_index_double_asc => indoc!(
	"
	INSERT INTO db_db_data.kv_index_double_asc (database_id, collection, \"index\", group_by, entry_id, rank_0, entry)
	VALUES (?, ?, ?, ?, ?, ?, ?)
	"
));

rivet_pools::cass_prepared_statement!(insert_entry_index_text_asc => indoc!(
	"
	INSERT INTO db_db_data.kv_index_text_asc (database_id, collection, \"index\", group_by, entry_id, rank_0, entry)
	VALUES (?, ?, ?, ?, ?, ?, ?)
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
					let collection = collection.clone();
					move |entry| {
						let cass_data = cass_data.clone();

						GlobalResult::Ok(insert_entry(
							cass_data,
							database_id,
							collection.clone(),
							entry.value,
						))
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

async fn insert_entry(
	cass_data: CassPool,
	database_id: Uuid,
	collection: backend::db::Collection,
	value_str: String,
) -> GlobalResult<common::Uuid> {
	let entry_id = Uuid::new_v4();

	// TODO: Do this in batch?

	// Deserialize value
	let value = serde_json::from_str::<JsonValue>(&value_str)?;
	let value_bson = bson::to_vec(&value)?;

	// Insert primary
	cass_data
		.execute(
			insert_entry::prepare(&cass_data).await?,
			(database_id, &collection.name_id, entry_id, &value_bson),
		)
		.await?;

	// Inset indexes
	for index in &collection.indexes {
		// Build group by map
		let mut group_by_map = HashMap::<Vec<String>, String>::new();
		for group_by in &index.group_by {
			let group_key = internal_unwrap!(group_by.field_path).field_path.clone();
			let group_value = lookup_field_path(&value, internal_unwrap!(group_by.field_path))?;
			let group_value_str = stringify_single_json_value_deterministic(group_value)?;
			group_by_map.insert(group_key, group_value_str);
		}

		// Determine if we should also insert the entry's value
		let entry = if index.include_entry {
			MaybeUnset::Set(&value_bson)
		} else {
			MaybeUnset::Unset
		};

		// Run query
		let d = database_id;
		let c = &collection.name_id;
		let i = &index.name_id;
		let g = &group_by_map;
		let ei = entry_id;
		let e = entry;
		if index.order_by.is_empty() {
			cass_data
				.execute(
					insert_entry_index::prepare(&cass_data).await?,
					(d, c, i, g, ei, e),
				)
				.await?;
		} else if index.order_by.len() == 1 {
			let order_0 = &index.order_by[0];
			let order_0_fp = internal_unwrap!(order_0.field_path);
			let order_0_v = lookup_field_path(&value, order_0_fp)?;

			match (
				internal_unwrap!(OrderFieldType::from_i32(order_0.field_type)),
				internal_unwrap!(OrderDir::from_i32(order_0.direction)),
			) {
				(OrderFieldType::Int, OrderDir::Asc) => {
					let r = internal_unwrap_owned!(order_0_v.as_i64(), "rank not i64");
					cass_data
						.execute(
							insert_entry_index_bigint_asc::prepare(&cass_data).await?,
							(d, c, i, g, ei, r, e),
						)
						.await?;
				}
				(OrderFieldType::Float, OrderDir::Asc) => {
					let r = internal_unwrap_owned!(order_0_v.as_f64(), "rank not f64");
					cass_data
						.execute(
							insert_entry_index_double_asc::prepare(&cass_data).await?,
							(d, c, i, g, ei, r, e),
						)
						.await?;
				}
				(OrderFieldType::String, OrderDir::Asc) => {
					let r = internal_unwrap_owned!(order_0_v.as_str(), "rank not f64");
					cass_data
						.execute(
							insert_entry_index_text_asc::prepare(&cass_data).await?,
							(d, c, i, g, ei, r, e),
						)
						.await?;
				}
				_ => internal_panic!("todo"),
			}
		} else {
			internal_panic!("unreachable")
		}
	}

	GlobalResult::Ok(common::Uuid::from(entry_id))
}

fn lookup_field_path<'a>(
	value: &'a JsonValue,
	field_path: &'a FieldPath,
) -> GlobalResult<&'a JsonValue> {
	lookup_field_path_inner(value, field_path.field_path.as_slice())
}

fn lookup_field_path_inner<'a>(
	value: &'a JsonValue,
	field_path: &'a [String],
) -> GlobalResult<&'a JsonValue> {
	if field_path.is_empty() {
		return Ok(value);
	}
	let field = &field_path[0];
	let field_path = &field_path[1..];
	match value {
		JsonValue::Object(obj) => {
			let value = internal_unwrap_owned!(obj.get(field));
			lookup_field_path_inner(value, field_path)
		}
		JsonValue::Array(arr) => {
			let index = field.parse::<usize>()?;
			let value = internal_unwrap_owned!(arr.get(index));
			lookup_field_path_inner(value, field_path)
		}
		_ => internal_panic!("field not found"),
	}
}

/// Converts a JSON value to string and throws error if it's a collection or non-deterministic
/// (e.g. floating point number).
fn stringify_single_json_value_deterministic(value: &JsonValue) -> GlobalResult<String> {
	internal_assert!(
		matches!(
			value,
			JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_)
		),
		"unsupported group by type"
	);
	if let JsonValue::Number(n) = value {
		internal_assert!(
			n.is_i64() || n.is_u64(),
			"cannot use floating point numbers as group by keys"
		);
	}

	let value_str = serde_json::to_string(value)?;

	Ok(value_str)
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
