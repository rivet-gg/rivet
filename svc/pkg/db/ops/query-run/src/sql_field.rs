use proto::backend;
use rivet_operation::prelude::*;
use sqlx::Row;
use util_db::{ais, entry_id::EntryId};

/// Helps with querying SQL fields.
///
/// Abstracts away internal columns (which have custom names) with the user-defined columns.
pub enum SqlField {
	Id,
	UserDefined {
		name_id: String,
		r#type: backend::db::field::Type,
		optional: bool,
	},
}

impl SqlField {
	/// Creates a new SQL field from a user-defined field or matches internal fields.
	pub fn from_user_defined_or_internal(
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
	pub fn from_user_defined_name_id(
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
	pub fn from_user_defined(field: &backend::db::Field) -> GlobalResult<Self> {
		Ok(Self::UserDefined {
			name_id: field.name_id.clone(),
			r#type: internal_unwrap_owned!(backend::db::field::Type::from_i32(field.r#type)),
			optional: field.optional,
		})
	}
}

impl SqlField {
	/// Name ID exposed to the user.
	pub fn field_name_id(&self) -> &str {
		match self {
			SqlField::Id => "_id",
			SqlField::UserDefined { name_id, .. } => &name_id,
		}
	}

	pub fn field_type(&self) -> backend::db::field::Type {
		match self {
			SqlField::Id => backend::db::field::Type::String,
			SqlField::UserDefined { r#type, .. } => *r#type,
		}
	}

	pub fn field_optional(&self) -> bool {
		match self {
			SqlField::Id => false,
			SqlField::UserDefined { optional, .. } => *optional,
		}
	}

	pub fn sql_column_name(&self) -> String {
		match self {
			SqlField::Id => "id".into(),
			SqlField::UserDefined { name_id, .. } => util_db::column_name(&name_id),
		}
	}

	/// Pushes the SQL column name.
	pub fn push_column_name(
		&self,
		builder: &mut sqlx::QueryBuilder<sqlx::Postgres>,
	) -> GlobalResult<()> {
		builder.push(format!(r#""{}""#, ais(&self.sql_column_name())?));
		Ok(())
	}

	/// Validates a given value can be written to a given field.
	pub fn bind_value(
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

				let id = EntryId::decode(&x)?;
				builder.push_bind(id.entry_id);
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
	pub fn get_column(
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
				VT::String(EntryId::new(id_raw).encode()?)
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
