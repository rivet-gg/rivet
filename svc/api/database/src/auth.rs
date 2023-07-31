use api_helper::{
	auth::{ApiAuth, AuthRateLimitCtx},
	util::{as_auth_expired, basic_rate_limit},
};
use proto::{backend::pkg::*, claims::Claims};
use rivet_claims::ClaimsDecode;
use rivet_operation::prelude::*;
use std::collections::HashSet;

/// Information derived from the authentication middleware.
pub struct Auth {
	claims: Option<Claims>,
}

#[async_trait]
impl ApiAuth for Auth {
	async fn new(
		api_token: Option<String>,
		rate_limit_ctx: AuthRateLimitCtx<'_>,
	) -> GlobalResult<Auth> {
		Self::rate_limit(rate_limit_ctx).await?;

		Ok(Auth {
			claims: if let Some(api_token) = api_token {
				Some(as_auth_expired(rivet_claims::decode(&api_token)?)?)
			} else {
				None
			},
		})
	}

	async fn rate_limit(rate_limit_ctx: AuthRateLimitCtx<'_>) -> GlobalResult<()> {
		basic_rate_limit(rate_limit_ctx).await
	}
}

impl Auth {
	pub fn claims(&self) -> GlobalResult<&Claims> {
		self.claims
			.as_ref()
			.ok_or_else(|| err_code!(API_UNAUTHORIZED))
	}

	pub async fn database(
		&self,
		ctx: &OperationContext<()>,
		database_id: Option<Uuid>,
	) -> GlobalResult<Uuid> {
		let claims = self.claims()?;

		let database_id = if let Some(database_id) = database_id {
			if let Some(user) = claims.as_user_option()? {
				// Get database owner team
				let database_res = op!([ctx] db_get {
					database_ids: vec![database_id.into()],
				})
				.await?;
				let database = internal_unwrap_owned!(database_res.databases.first());
				let db_team_id = internal_unwrap!(database.owner_team_id).as_uuid();

				// Check if user is a member of the team
				let member_res = op!([ctx] team_member_get {
					members: vec![team::member_get::request::TeamMember {
						team_id: Some(db_team_id.into()),
						user_id: Some(user.user_id.into()),
					}]
				})
				.await?;
				assert_with!(member_res.members.len() == 1, API_UNAUTHORIZED);

				database_id
			} else if let Some(game) = claims.as_game_cloud_option()? {
				// Get namespace IDs associated with this token
				let ns_res = op!([ctx] game_namespace_list {
					game_ids: vec![game.game_id.into()],
				})
				.await?;
				let namespace_ids = internal_unwrap_owned!(ns_res.games.first())
					.namespace_ids
					.clone();

				// Get the version IDs associated with these namespaces
				let ns_res = op!([ctx] game_namespace_get {
					namespace_ids: namespace_ids,
				})
				.await?;
				let version_ids = ns_res
					.namespaces
					.iter()
					.flat_map(|x| x.version_id)
					.map(|x| x.as_uuid())
					.collect::<HashSet<Uuid>>()
					.into_iter()
					.map(common::Uuid::from)
					.collect::<Vec<_>>();

				// Get the database IDs associated with these versions
				let db_version_res = op!([ctx] db_game_version_get {
					version_ids: version_ids,
				})
				.await?;

				// Check if one of the versions for this game uses the given database
				let version_matches = db_version_res
					.versions
					.iter()
					.filter_map(|x| x.config_meta.as_ref())
					.filter_map(|x| x.database_id)
					.any(|x| x.as_uuid() == database_id);
				assert_with!(version_matches, API_UNAUTHORIZED);

				database_id
			} else {
				panic_with!(API_UNAUTHORIZED)
			}
		} else if let Some(game_ns_dev) = claims.as_game_namespace_development_option()? {
			self.database_for_namespace(ctx, game_ns_dev.namespace_id)
				.await?
		} else if let Ok(lobby_ent) = claims.as_matchmaker_lobby() {
			// Get lobby's namespace ID
			let lobbies_res = op!([ctx] mm_lobby_get {
				lobby_ids: vec![lobby_ent.lobby_id.into()],
				include_stopped: false,
			})
			.await?;
			let lobby = internal_unwrap_owned!(lobbies_res.lobbies.first());
			let namespace_id = internal_unwrap!(lobby.namespace_id).as_uuid();

			self.database_for_namespace(ctx, namespace_id).await?
		} else {
			panic_with!(API_UNAUTHORIZED);
		};

		Ok(database_id)
	}

	async fn database_for_namespace(
		&self,
		ctx: &OperationContext<()>,
		namespace_id: Uuid,
	) -> GlobalResult<Uuid> {
		// Get namespace's version ID
		let namespaces_res = op!([ctx] game_namespace_get {
			namespace_ids: vec![namespace_id.into()],
		})
		.await?;
		let version_id =
			internal_unwrap!(internal_unwrap_owned!(namespaces_res.namespaces.first()).version_id)
				.as_uuid();

		// Get version's database ID
		let versions_res = op!([ctx] db_game_version_get {
			version_ids: vec![version_id.into()],
		})
		.await?;
		let version = unwrap_with_owned!(
			versions_res.versions.first(),
			API_FORBIDDEN,
			reason = "Database service not enabled for this namespace"
		);
		let database_id =
			internal_unwrap!(internal_unwrap!(version.config_meta).database_id).as_uuid();

		Ok(database_id)
	}
}
