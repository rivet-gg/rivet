use global_error::prelude::*;

pub mod entry_id;

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
