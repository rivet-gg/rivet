use global_error::prelude::*;

// ud = user defined

pub fn schema_name(database_id_short: &str) -> String {
	format!("ud_{database_id_short}")
}

pub fn table_name(name_id: &str) -> String {
	format!("ud_{name_id}")
}

pub fn column_name(name_id: &str) -> String {
	format!("ud_{name_id}")
}

/// Converts ID from database -> user-friendly.
///
/// IDs are encoded in order to:
/// 1. Include extra data in the ID (like the database region).
/// 2. Prevent users from guessing other IDs.
/// 3. Prevent users from guessing the number of records in the database.
pub fn encode_id(id: i64) -> GlobalResult<String> {
	Ok(format!("xxx{}", id.to_string()))
}

/// Converts ID from user-friendly -> database.
///
/// See `encode_id` for more details.
pub fn decode_id(id: &str) -> GlobalResult<i64> {
	Ok(id[3..].parse::<i64>()?)
}

/// Validates this is a safe identifier and returns error if not.
///
/// This is a redundant check to the previous ident checks in `merge_schemas`.
pub fn assert_ident_snake(x: &str) -> GlobalResult<&str> {
	internal_assert!(
		rivet_util::check::ident_snake(x),
		"unhandled invalid identifier"
	);
	Ok(x)
}

// Short alias since this is used frequently.
pub use assert_ident_snake as ais;
