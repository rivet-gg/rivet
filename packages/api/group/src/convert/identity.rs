use rivet_group_server::models;
use rivet_operation::prelude::*;

pub fn handle(
	config: &rivet_config::Config,
	_current_user_id: Uuid,
	user: &user::types::User,
) -> GlobalResult<models::IdentityHandle> {
	Ok(models::IdentityHandle {
		identity_id: user.user_id.to_string(),
		display_name: user.display_name.clone(),
		account_number: user.account_number as i32,
		avatar_url: user::route::user_avatar(config, user),
		is_registered: true, // TODO:
		external: models::IdentityExternalLinks {
			profile: user::route::user_profile(config, user.user_id),
			settings: None,
			chat: Default::default(),
		},
		party: None,
		presence: None,
	})
}
