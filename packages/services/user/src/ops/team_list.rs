use chirp_workflow::prelude::*;

#[derive(Debug)]
pub struct Input {
	pub user_ids: Vec<Uuid>,
}

#[derive(Debug)]
pub struct Output {
	pub users: Vec<UserTeams>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserTeams {
	pub user_id: Uuid,
	pub teams: Vec<TeamMember>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TeamMember {
    pub team_id: Uuid,
}

#[operation]
pub async fn team_list(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	let users = ctx
		.cache()
		.fetch_all_json("user_team_list", input.user_ids.clone(), {
			let ctx = ctx.clone();
			move |mut cache, user_ids| {
				let ctx = ctx.clone();
				async move {
					let team_members = sql_fetch_all!(
						[ctx, (Uuid, Uuid)]
						"
					SELECT user_id, team_id
					FROM db_team.team_members
					WHERE user_id = ANY($1)
					",
						&user_ids,
					)
					.await?;

					for user_id in user_ids {
						// Aggregate user teams
						let user_teams = UserTeams {
							user_id: user_id,
							teams: team_members
								.iter()
								.filter(|(team_user_id, _)| *team_user_id == user_id)
								.map(|(_, team_id)| TeamMember {
									team_id: *team_id,
								})
								.collect(),
						};

						cache.resolve(&user_id.clone(), user_teams);
					}

					Ok(cache)
				}
			}
		})
		.await?;

	Ok(Output { users })
}
