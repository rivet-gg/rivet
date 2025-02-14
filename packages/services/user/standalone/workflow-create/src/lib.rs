use chirp_workflow::prelude::*;

const USER_BATCH_SIZE: usize = 128;

#[tracing::instrument(skip_all)]
pub async fn start(config: rivet_config::Config, pools: rivet_pools::Pools) -> GlobalResult<()> {
	let client =
		chirp_client::SharedClient::from_env(pools.clone())?.wrap_new("user-workflow-create");
	let cache = rivet_cache::CacheInner::from_env(pools.clone())?;
	let ctx = StandaloneCtx::new(
		chirp_workflow::compat::db_from_pools(&pools).await?,
		config,
		rivet_connection::Connection::new(client, pools, cache),
		"user-workflow-create",
	)
	.await?;

	let (user_count,) = sql_fetch_one!(
		[ctx, (i64,)]
		"
		SELECT COUNT(*)
		FROM db_users.users
		WHERE EXISTS(
			SELECT 1
			FROM db_user_identity.emails as e
			WHERE e.user_id = users.user_id
		)
		AND NOT EXISTS(
			SELECT 1
			FROM db_workflow.workflows
			WHERE
				workflow_name = 'user' AND
				(tags->>'user_id')::UUID = users.user_id
		)
		",
	)
	.await?;

	if user_count == 0 {
		return Ok(())
	}

	for offset in (0..user_count).step_by(USER_BATCH_SIZE) {
		tracing::debug!(?offset, "creating users");

		let user_ids = sql_fetch_all!(
			[ctx, (Uuid,)]
			"
			SELECT user_id
			FROM db_user.users
			WHERE EXISTS(
				SELECT 1
				FROM db_user_identity.emails as e
				WHERE e.user_id = users.user_id
			)
			AND NOT EXISTS(
				SELECT 1
				FROM db_workflow.workflows
				WHERE
					workflow_name = 'user' AND
					(tags->>'user_id')::UUID = users.user_id
			)
			LIMIT $1 OFFSET $2
			",
			offset,
			USER_BATCH_SIZE as i64
		)
		.await?
		.into_iter()
		.map(|(user_id,)| user_id)
		.collect::<Vec<_>>();

		if user_ids.len() == 0 {
			continue;
		}

		for user_id in user_ids {
			let mut sub = ctx.subscribe::<
				user::workflows::user::CreateComplete
			>(("user_id", user_id)).await?;

			let _ = ctx.workflow(user::workflows::user::Input {
				user_id,
				display_name: None,
				is_already_in_db: true
			})
			.tag("user_id", user_id)
			.dispatch();

			// Await creation completion
			sub.next().await?;
		}
	}

	Ok(())
}
