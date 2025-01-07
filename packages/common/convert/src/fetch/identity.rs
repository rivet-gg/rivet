
use rivet_operation::prelude::proto;
use proto::{backend::{self}, common};
use rivet_api::models;
use chirp_workflow::prelude::*;

use crate::convert;

#[derive(Debug)]
pub struct TeamsCtx {
	pub user_teams: user::ops::team_list::Output,
	pub teams: Vec<backend::team::Team>,
}

pub async fn handles(
	ctx: &ApiCtx,
	current_user_id: Uuid,
	user_ids: Vec<Uuid>,
) -> GlobalResult<Vec<models::IdentityHandle>> {
	if user_ids.is_empty() {
		return Ok(Vec::new());
	}

	let users = users(ctx, user_ids.clone()).await?;

	// Convert all data
	users
		.users
		.iter()
		.map(|user| convert::identity::handle(ctx.config(), current_user_id, user))
		.collect::<GlobalResult<Vec<_>>>()
}

pub async fn summaries(
	ctx: &ApiCtx,
	current_user_id: Uuid,
	user_ids: Vec<Uuid>,
) -> GlobalResult<Vec<models::IdentitySummary>> {
	if user_ids.is_empty() {
		return Ok(Vec::new());
	}

	let users = users(ctx, user_ids.clone()).await?;

	// Convert all data
	users
		.users
		.iter()
		.map(|user| convert::identity::summary(ctx.config(), current_user_id, user))
		.collect::<GlobalResult<Vec<_>>>()
}

pub async fn profiles(
	ctx: &ApiCtx,
	current_user_id: Uuid,
	raw_user_ids: Vec<Uuid>,
) -> GlobalResult<Vec<models::IdentityProfile>> {
	if raw_user_ids.is_empty() {
		return Ok(Vec::new());
	}

	let (users, teams_ctx, linked_accounts) = tokio::try_join!(
		users(ctx, raw_user_ids.clone()),
		teams(ctx, raw_user_ids.clone()),
		linked_accounts(ctx, raw_user_ids.clone()),
	)?;

	// Convert all data
	users
		.users
		.iter()
		.map(|user| {
			convert::identity::profile(
				ctx.config(),
				current_user_id,
				user,
				convert::identity::ProfileCtx {
					teams_ctx: &teams_ctx,
					linked_accounts: &linked_accounts.users,
					self_is_game_linked: false,
				},
			)
		})
		.collect::<GlobalResult<Vec<_>>>()
}

pub async fn users(
	ctx: &ApiCtx,
	user_ids: Vec<Uuid>,
) -> GlobalResult<user::ops::get::Output> {
	ctx.op(user::ops::get::Input {
		user_ids,
	})
	.await
}

async fn teams(ctx: &ApiCtx, user_ids: Vec<Uuid>) -> GlobalResult<TeamsCtx> {
	let user_teams_res = ctx.op(
		user::ops::team_list::Input {
			user_ids,
		},
	)
	.await?;

	let team_ids = user_teams_res
		.users
		.iter()
		.map(|user| {
			user.teams
				.iter()
				.map(|t| Ok(common::Uuid::from(t.team_id)))
				.collect::<GlobalResult<Vec<_>>>()
		})
		.collect::<GlobalResult<Vec<_>>>()?
		.into_iter()
		.flatten()
		.collect::<Vec<_>>();

	let teams_res = op!([ctx] team_get {
		team_ids: team_ids.clone(),
	})
	.await?;

	// TODO: hide all closed teams
	let teams = teams_res.teams.clone();

	Ok(TeamsCtx {
		user_teams: user_teams_res,
		teams,
	})
}

async fn linked_accounts(
	ctx: &ApiCtx,
	user_ids: Vec<Uuid>,
) -> GlobalResult<user::ops::identity::get::Output> {
	Ok(
		ctx.op(user::ops::identity::get::Input {
			user_ids,
		})
		.await?
	)
}
