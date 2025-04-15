use std::collections::HashMap;

use chirp_workflow::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Route {
	pub route_id: Uuid,
	pub namespace_id: Uuid,
	pub name_id: String,
	pub hostname: String,
	pub path: String,
	pub route_subpaths: bool,
	pub selector_tags: HashMap<String, String>,
	pub create_ts: i64,
	pub update_ts: i64,
	pub delete_ts: Option<i64>,
}
