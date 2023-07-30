use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
use sqlx::Row;
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
				.map(|x| encode_id(x.0))
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

/// Represents an SQL field to query.
enum SqlField {
	Id,
	UserDefined {
		name_id: String,
		r#type: backend::db::field::Type,
		optional: bool,
	},
}

impl SqlField {
	/// Creates a new SQL field from a user-defined field or matches internal fields.
	fn from_user_defined_or_internal(
		collection: &backend::db::Collection,
		name_id: &str,
	) -> GlobalResult<Self> {
		if name_id == "_id" {
			Ok(Self::Id)
		} else if name_id.starts_with("_") {
			// This is an invalid field, since all internal fields start with "_"
			panic_with!(DB_FIELD_NOT_FOUND, field = name_id)
		} else {
			Self::from_user_defined_name_id(collection, name_id)
		}
	}

	/// Finds a user-defined field in a collection. Does not match internal fields.
	fn from_user_defined_name_id(
		collection: &backend::db::Collection,
		name_id: &str,
	) -> GlobalResult<Self> {
		// Find existing field
		let field = unwrap_with_owned!(
			collection.fields.iter().find(|x| x.name_id == name_id),
			DB_FIELD_NOT_FOUND,
			field = name_id
		);
		Ok(Self::from_user_defined(field)?)
	}

	/// Creates a new user-defined field.
	fn from_user_defined(field: &backend::db::Field) -> GlobalResult<Self> {
		Ok(Self::UserDefined {
			name_id: field.name_id.clone(),
			r#type: internal_unwrap_owned!(backend::db::field::Type::from_i32(field.r#type)),
			optional: field.optional,
		})
	}
}

impl SqlField {
	/// Name ID exposed to the user.
	fn field_name_id(&self) -> &str {
		match self {
			SqlField::Id => "_id",
			SqlField::UserDefined { name_id, .. } => &name_id,
		}
	}

	fn field_type(&self) -> backend::db::field::Type {
		match self {
			SqlField::Id => backend::db::field::Type::String,
			SqlField::UserDefined { r#type, .. } => *r#type,
		}
	}

	fn field_optional(&self) -> bool {
		match self {
			SqlField::Id => false,
			SqlField::UserDefined { optional, .. } => *optional,
		}
	}

	fn sql_column_name(&self) -> String {
		match self {
			SqlField::Id => "id".into(),
			SqlField::UserDefined { name_id, .. } => util_db::column_name(&name_id),
		}
	}

	/// Pushes the SQL column name.
	fn push_column_name(
		&self,
		builder: &mut sqlx::QueryBuilder<sqlx::Postgres>,
	) -> GlobalResult<()> {
		builder.push(format!(r#""{}""#, ais(&self.sql_column_name())?));
		Ok(())
	}

	/// Validates a given value can be written to a given field.
	fn bind_value(
		&self,
		builder: &mut sqlx::QueryBuilder<sqlx::Postgres>,
		value: &backend::db::Value,
	) -> GlobalResult<()> {
		use backend::db::field::Type as FT;
		use backend::db::value::Type as VT;

		let value_type = internal_unwrap!(value.r#type);
		match (self, self.field_type(), value_type) {
			// Custom field encoders
			(Self::Id, ft, VT::String(x)) => {
				internal_assert_eq!(FT::String, ft, "unexpected field type");

				let id = decode_id(&x)?;
				builder.push_bind(id);
			}

			// Generic types
			(_, FT::Int, VT::Int(x)) => {
				builder.push_bind(*x);
			}
			(_, FT::Float, VT::Float(x)) => {
				builder.push_bind(*x);
			}
			(_, FT::Bool, VT::Bool(x)) => {
				builder.push_bind(*x);
			}
			(_, FT::String, VT::String(x)) => {
				builder.push_bind(x.clone());
			}
			(_, _, VT::Null(_)) => {
				if self.field_optional() {
					builder.push_bind(Option::<i64>::None);
				} else {
					panic_with!(
						DB_CANNOT_ASSIGN_NULL_TO_NON_OPTIONAL,
						field = self.field_name_id()
					);
				}
			}
			_ => panic_with!(
				DB_VALUE_DOES_NOT_MATCH_FIELD_TYPE,
				field = self.field_name_id()
			),
		};

		Ok(())
	}

	/// Reads a column from a raw row based on the field type.
	fn get_column(
		&self,
		row: &sqlx::postgres::PgRow,
		i: usize,
	) -> GlobalResult<backend::db::Value> {
		use backend::db::field::Type as FT;
		use backend::db::value::Type as VT;

		let value_ty = match (self, self.field_type()) {
			// Custom field decoders
			(Self::Id, ft) => {
				internal_assert_eq!(FT::String, ft, "unexpected field type");

				let id_raw = row.try_get::<i64, _>(i)?;
				VT::String(encode_id(id_raw)?)
			}

			// Generic types
			(_, FT::Int) => row
				.try_get::<Option<i64>, _>(i)?
				.map_or_else(|| VT::Null(Default::default()), VT::Int),
			(_, FT::Float) => row
				.try_get::<Option<f64>, _>(i)?
				.map_or_else(|| VT::Null(Default::default()), VT::Float),
			(_, FT::Bool) => row
				.try_get::<Option<bool>, _>(i)?
				.map_or_else(|| VT::Null(Default::default()), VT::Bool),
			(_, FT::String) => row
				.try_get::<Option<String>, _>(i)?
				.map_or_else(|| VT::Null(Default::default()), VT::String),
		};

		Ok(backend::db::Value {
			r#type: Some(value_ty),
		})
	}
}

// TODO: Write encoders
fn decode_id(id: &str) -> GlobalResult<i64> {
	Ok(id[3..].parse::<i64>()?)
}

fn encode_id(id: i64) -> GlobalResult<String> {
	Ok(format!("xxx{}", id.to_string()))
}
