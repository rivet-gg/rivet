use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
use std::collections::HashMap;
use util_db::ais;

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
	let inserted_entries = run_query(&pg_data, &database_id_short, &schema, query).await?;

	Ok(db::query_run::Response {
		inserted_entries: inserted_entries
			.into_iter()
			.map(|x| x.to_string())
			.collect::<Vec<_>>(),
	})
}

async fn run_query(
	pg_data: &PostgresPool,
	database_id_short: &str,
	schema: &backend::db::Schema,
	query: &backend::db::Query,
) -> GlobalResult<Vec<i64>> {
	let schema_name = util_db::schema_name(database_id_short);

	// TODO: Do bulk inserts with https://github.com/launchbadge/sqlx/blob/main/FAQ.md#how-can-i-bind-an-array-to-a-values-clause-how-can-i-do-bulk-inserts

	match internal_unwrap!(query.kind) {
		backend::db::query::Kind::Get(get) => {
			todo!()
		}
		backend::db::query::Kind::Insert(set) => {
			let collection = get_collection(schema, &set.collection)?;

			let table = util_db::table_name(&collection.name_id);
			let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new(format!(
				r#"INSERT INTO "{schema}"."{table}" ("#,
				schema = ais(&schema_name)?,
				table = ais(&table)?,
			));

			// Specify columns
			for (i, key) in set.fields.keys().enumerate() {
				let field = get_field(collection, key)?;
				query.push(ais(&field.name_id)?);
				if i != set.fields.len() - 1 {
					query.push(", ");
				}
			}

			// Bind values
			query.push(") VALUES (");
			for (i, (key, value)) in set.fields.iter().enumerate() {
				let field = get_field(collection, key)?;
				bind_value_for_field(&mut query, field, value)?;
				if i != set.fields.len() - 1 {
					query.push(", ");
				}
			}
			query.push(") RETURNING (id)");

			tracing::info!(sql = ?query.sql(), "running insert");

			let inserted_entries = query
				.build_query_as::<(i64,)>()
				.fetch_all(pg_data)
				.await?
				.into_iter()
				.map(|x| x.0)
				.collect();

			Ok(inserted_entries)
		}
	}
}

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

fn get_field<'a>(
	collection: &'a backend::db::Collection,
	field: &str,
) -> GlobalResult<&'a backend::db::Field> {
	let x = unwrap_with_owned!(
		collection.fields.iter().find(|x| x.name_id == field),
		DB_FIELD_NOT_FOUND,
		field = field
	);
	Ok(x)
}

/// Validates a given value can be written to a given field.
fn bind_value_for_field(
	builder: &mut sqlx::QueryBuilder<sqlx::Postgres>,
	field: &backend::db::Field,
	value: &backend::db::Value,
) -> GlobalResult<()> {
	use backend::db::field::Type as FT;
	use backend::db::value::Type as VT;

	let field_type = internal_unwrap_owned!(FT::from_i32(field.r#type));
	let value_type = internal_unwrap!(value.r#type);
	match (field_type, value_type) {
		(FT::Int, VT::Int(x)) => {
			builder.push_bind(*x);
		}
		(FT::Float, VT::Float(x)) => {
			builder.push_bind(*x);
		}
		(FT::Bool, VT::Bool(x)) => {
			builder.push_bind(*x);
		}
		(FT::String, VT::String(x)) => {
			builder.push_bind(x.clone());
		}
		(_, VT::Null(_)) => {
			if field.optional {
				builder.push_bind(Option::<i64>::None);
			} else {
				panic_with!(DB_CANNOT_ASSIGN_NULL_TO_NON_OPTIONAL, field = field.name_id);
			}
		}
		_ => panic_with!(DB_VALUE_DOES_NOT_MATCH_FIELD_TYPE, field = field.name_id),
	};

	Ok(())
}
