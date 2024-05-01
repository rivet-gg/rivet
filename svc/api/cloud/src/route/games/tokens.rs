use api_helper::ctx::Ctx;
use rivet_api::models;
use rivet_operation::prelude::{proto::backend::pkg::token, *};

use crate::auth::Auth;

// Also see user-token-create/src/main.rs
pub const TOKEN_TTL: i64 = util::duration::minutes(15);
pub const REFRESH_TOKEN_TTL: i64 = util::duration::days(90);

// MARK: POST /games/{}/tokens/cloud
pub async fn create_cloud_token(
	ctx: Ctx<Auth>,
	game_id: Uuid,
	_body: serde_json::Value,
) -> GlobalResult<models::CloudGamesCreateCloudTokenResponse> {
	ctx.auth()
		.check_game_write_or_admin(ctx.op_ctx(), game_id)
		.await?;

	let token_res = op!([ctx] cloud_game_token_create {
		game_id: Some(game_id.into()),
	})
	.await?;

	Ok(models::CloudGamesCreateCloudTokenResponse {
		token: token_res.token,
	})
}

// MARK: POST /games/{}/tokens/service
pub async fn create_service_token(
	ctx: Ctx<Auth>,
	game_id: Uuid,
	_body: serde_json::Value,
) -> GlobalResult<models::CloudGamesCreateServiceTokenResponse> {
	ctx.auth()
		.check_game_write_or_admin(ctx.op_ctx(), game_id)
		.await?;

	let token_res = op!([ctx] token_create {
		token_config: Some(token::create::request::TokenConfig {
			ttl: TOKEN_TTL,
		}),
		refresh_token_config: Some(token::create::request::TokenConfig {
			ttl: REFRESH_TOKEN_TTL,
		}),
		issuer: "api-cloud".to_owned(),
		client: Some(ctx.client_info()),
		kind: Some(token::create::request::Kind::New(
			token::create::request::KindNew { entitlements: vec![proto::claims::Entitlement {
				kind: Some(proto::claims::entitlement::Kind::GameService(
					proto::claims::entitlement::GameService {
						game_id: Some(game_id.into()),
					}
				)),
			}]},
		)),
		label: Some("game_service".to_owned()),
		..Default::default()
	})
	.await?;

	Ok(models::CloudGamesCreateServiceTokenResponse {
		token: unwrap!(token_res.token).token,
	})
}
