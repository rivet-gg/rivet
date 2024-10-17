use rivet_config::Config;
use std::collections::HashMap;
use tokio::task::JoinSet;

use crate::Error;

pub type RedisPool = redis::aio::ConnectionManager;

#[tracing::instrument(skip(config))]
pub async fn setup(config: Config) -> Result<HashMap<String, RedisPool>, Error> {
	// Create Redis connections
	let mut join_set = JoinSet::new();
	let redis_types = &config.server().map_err(Error::Global)?.redis;
	for (key, redis_config) in [
		("ephemeral", redis_types.ephemeral.clone()),
		("persistent", redis_types.persistent.clone()),
	] {
		join_set
			.build_task()
			.name("redis_from_env")
			.spawn(async move {
				tracing::info!(url = %redis_config.url, "redis connecting");
				let client =
					redis::Client::open(redis_config.url.as_str()).map_err(Error::BuildRedis)?;
				let conn =
					redis::aio::ConnectionManager::new_with_backoff(client, 2, 100, usize::MAX)
						.await
						.map_err(Error::BuildRedis)?;

				tracing::info!(url = %redis_config.url, "redis connected");

				Ok((key, conn))
			})
			.map_err(Error::TokioSpawn)?;
	}

	// Join connections
	let mut redis = HashMap::new();
	while let Some(res) = join_set.join_next().await {
		let (key, conn) = res.map_err(Error::TokioJoin)??;
		redis.insert(key.to_string(), conn.clone());
	}

	Ok(redis)
}
