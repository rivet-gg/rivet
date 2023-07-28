use global_error::prelude::*;
use uuid::Uuid;

pub fn schema_name(database_id: Uuid) -> String {
	format!("data_{}", database_id.to_string().replace("-", "_"))
}

pub fn table_name(name_id: &str) -> String {
	format!("data_{name_id}")
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
