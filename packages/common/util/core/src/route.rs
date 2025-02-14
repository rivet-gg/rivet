use types_proto::rivet::backend;
use uuid::Uuid;

// TODO: Remove unwraps of server config

pub fn user_settings(config: &rivet_config::Config) -> String {
	format!(
		"{}/settings",
		crate::url::to_string_without_slash(&config.server().unwrap().rivet.ui.public_origin())
	)
}

pub fn team_profile(config: &rivet_config::Config, team_id: Uuid) -> String {
	format!(
		"{}/groups/{}",
		crate::url::to_string_without_slash(&config.server().unwrap().rivet.ui.public_origin()),
		team_id
	)
}

pub fn custom_avatar(
	config: &rivet_config::Config,
	upload_id: Uuid,
	file_name: &str,
	_provider: i32,
) -> String {
	format!(
		"{}/media/user-avatar/{}/{}",
		crate::url::to_string_without_slash(
			&config.server().unwrap().rivet.api_public.public_origin()
		),
		upload_id,
		file_name
	)
}

pub fn team_avatar(config: &rivet_config::Config, team: &backend::team::Team) -> Option<String> {
	if let (Some(upload_id), Some(file_name)) =
		(team.profile_upload_id, team.profile_file_name.as_ref())
	{
		Some(format!(
			"{}/media/team-avatar/{}/{}",
			crate::url::to_string_without_slash(
				&config.server().unwrap().rivet.api_public.public_origin()
			),
			upload_id,
			file_name
		))
	} else {
		None
	}
}

pub fn game_logo(config: &rivet_config::Config, game: &backend::game::Game) -> Option<String> {
	if let (Some(upload_id), Some(file_name)) = (game.logo_upload_id, game.logo_file_name.as_ref())
	{
		Some(format!(
			"{}/media/game-logo/{}/{}",
			crate::url::to_string_without_slash(
				&config.server().unwrap().rivet.api_public.public_origin()
			),
			upload_id,
			file_name
		))
	} else {
		None
	}
}

pub fn game_banner(config: &rivet_config::Config, game: &backend::game::Game) -> Option<String> {
	if let (Some(upload_id), Some(file_name)) =
		(game.banner_upload_id, game.banner_file_name.as_ref())
	{
		Some(format!(
			"{}/media/game-banner/{}/{}",
			crate::url::to_string_without_slash(
				&config.server().unwrap().rivet.api_public.public_origin()
			),
			upload_id,
			file_name
		))
	} else {
		None
	}
}

pub fn identity_game_link(config: &rivet_config::Config, link_token: &str) -> String {
	format!(
		"{}/link/{}",
		crate::url::to_string_without_slash(&config.server().unwrap().rivet.ui.public_origin()),
		link_token
	)
}

pub fn cloud_device_link(config: &rivet_config::Config, link_token: &str) -> String {
	format!(
		"{}/devices/link/{}",
		crate::url::to_string_without_slash(&config.server().unwrap().rivet.ui.public_origin()),
		link_token
	)
}

pub fn backend_endpoint(config: &rivet_config::Config, backend_slug: &str) -> String {
	let backend_domain = &config
		.server()
		.unwrap()
		.rivet
		.backend
		.as_ref()
		.expect("backend not enabled")
		.base_domain;

	format!("https://{}.{}", backend_slug, backend_domain)
}
