use chirp_workflow::prelude::*;
use proto::backend::{self, pkg::*};
use rivet_operation::prelude::proto;

// Also see api/auth/src/route/tokens.rs
pub const TOKEN_TTL: i64 = util::duration::minutes(15);
pub const REFRESH_TOKEN_TTL: i64 = util::duration::days(90);

#[derive(Debug)]
pub struct Input {
	pub user_id: Uuid,
	pub client: backend::net::ClientInfo,
}

#[derive(Debug)]
pub struct Output {
	pub token: String,
	pub refresh_token: String
}


#[operation]
pub async fn token_create(
    ctx: &OperationCtx,
    input: &Input
) -> GlobalResult<Output> {
    let user_id = input.user_id;

	let token_res = op!([ctx] token_create {
		token_config: Some(token::create::request::TokenConfig {
			ttl: TOKEN_TTL,
		}),
		refresh_token_config: Some(token::create::request::TokenConfig {
			ttl: REFRESH_TOKEN_TTL,
		}),
		issuer: Self::NAME.to_owned(),
		client: Some(input.client.clone()),
		kind: Some(token::create::request::Kind::New(
			token::create::request::KindNew {
				entitlements: vec![proto::claims::Entitlement {
					kind: Some(proto::claims::entitlement::Kind::User(proto::claims::entitlement::User {
						user_id: Some(user_id.into()),
					})),
				}],
			},
		)),
		label: Some("usr".into()),
		..Default::default()
	})
	.await?;

	let token = unwrap_ref!(token_res.token);
	let refresh_token = unwrap_ref!(token_res.refresh_token);
	let token_session_id = unwrap_ref!(token_res.session_id).as_uuid();

	sql_execute!(
		[ctx]
		"INSERT INTO db_user.user_tokens (user_id, token_session_id) VALUES ($1, $2)",
		user_id,
		token_session_id,
	)
	.await?;

	Ok(Output {
		token: token.token.clone(),
		refresh_token: refresh_token.token.clone(),
	})
}
