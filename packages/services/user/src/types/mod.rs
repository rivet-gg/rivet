use chirp_workflow::prelude::*;

pub mod identity;

#[derive(Clone, Debug, Default)]
pub struct User {
	pub user_id: Uuid,
	pub display_name: String,
	pub account_number: u32,
	pub avatar_id: String,
	pub profile_upload_id: Option<Uuid>,
    pub profile_file_name: Option<String>,
    pub profile_provider: Option<i32>, // backend::upload::Provider
	pub join_ts: i64,
	pub bio: String,
	pub is_admin: bool,
	pub delete_request_ts: Option<i64>,
	pub delete_complete_ts: Option<i64>,
}