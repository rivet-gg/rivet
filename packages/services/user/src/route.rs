use chirp_workflow::prelude::*;

pub fn user_avatar(config: &rivet_config::Config, user: &crate::types::User) -> String {
	if let (Some(upload_id), Some(file_name)) =
		(user.profile_upload_id, user.profile_file_name.as_ref())
	{
		format!(
			"{}/media/user-avatar/{}/{}",
			util::url::to_string_without_slash(
				&config.server().unwrap().rivet.api_public.public_origin()
			),
			upload_id,
			file_name
		)
	} else {
		format!("https://assets2.rivet.gg/avatars/{}.png", user.avatar_id)
	}
}

pub fn user_profile(config: &rivet_config::Config, user_id: Uuid) -> String {
	format!(
		"{}/identities/{}",
		util::url::to_string_without_slash(&config.server().unwrap().rivet.ui.public_origin()),
		user_id
	)
}
