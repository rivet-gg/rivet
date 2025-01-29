use futures_util::{StreamExt, TryStreamExt};
use indoc::indoc;
use chirp_workflow::prelude::*;


pub async fn start(config: rivet_config::Config, pools: rivet_pools::Pools) -> GlobalResult<()> {
	run_from_env(config, pools, util::timestamp::now()).await
}

#[tracing::instrument(skip_all)]
pub async fn run_from_env(
	config: rivet_config::Config,
	pools: rivet_pools::Pools,
	ts: i64,
) -> GlobalResult<()> {
	let client = chirp_client::SharedClient::from_env(pools.clone())?
		.wrap_new("user-delete-pending");
	let cache = rivet_cache::CacheInner::from_env(pools.clone())?;
	let ctx = StandaloneCtx::new(
		chirp_workflow::compat::db_from_pools(&pools).await?,
		config,
		rivet_connection::Connection::new(client, pools, cache),
		"user-delete-pending",
	)
	.await?;

	let user_ids = sql_fetch_all!(
		[ctx, (Uuid,)]
		"
		SELECT user_id
		FROM db_user.users
		WHERE delete_request_ts < $1
		",
		ts - util::duration::days(30),
	)
	.await?
	.into_iter()
	.map(|(user_id,)| user_id)
	.collect::<Vec<_>>();

	tracing::info!(count = user_ids.len(), "publishing deletes");

	futures_util::stream::iter(user_ids.into_iter())
		.map(|user_id| {
			let ctx = ctx.clone();
			async move {
				let mut sub = ctx.subscribe::<
					user::workflows::user::DeleteComplete
				>(("user_id", user_id)).await?;

				let _ = ctx.signal(user::workflows::user::Delete {})
					.tag("user_id", user_id)
					.send();
				
				// Await deletion completion
				sub.next().await?;

				Ok::<_, GlobalError>(())
			}
		})
		.buffer_unordered(32)
		.try_collect::<Vec<_>>()
		.await?;

	Ok(())
}
