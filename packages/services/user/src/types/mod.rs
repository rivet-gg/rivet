use chirp_workflow::prelude::*;
use rivet_operation::prelude::{proto};
use proto::backend;

pub mod identity;

#[derive(Debug, Default)]
pub struct User {
	pub user_id: Uuid,
	pub display_name: String,
	pub account_number: i64,
	pub avatar_id: String,
	pub profile_upload_id: Option<Uuid>,
    pub profile_file_name: Option<String>,
    pub profile_provider: Option<backend::upload::Provider>,
	pub join_ts: i64,
	pub bio: String,
	pub is_admin: bool,
	pub delete_request_ts: Option<i64>,
	pub delete_complete_ts: Option<i64>,
}