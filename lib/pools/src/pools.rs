use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::Error;

pub type NatsPool = async_nats::Client;
pub type CrdbPool = sqlx::PgPool;
pub type PostgresPool = sqlx::PgPool;
pub type CassandraPool = Arc<scylla::Session>;
pub type RedisPool = redis::aio::ConnectionManager;

pub type RedisConn = redis::aio::ConnectionManager;

pub type Pools = Arc<PoolsInner>;

pub struct PoolsInner {
	pub(crate) _guard: DropGuard,
	pub(crate) nats: Option<NatsPool>,
	pub(crate) crdb: HashMap<String, CrdbPool>,
	pub(crate) postgres: HashMap<String, PostgresPool>,
	pub(crate) cassandra: HashMap<String, CassandraPool>,
	pub(crate) redis: HashMap<String, RedisPool>,
}

impl PoolsInner {
	/// Spawn background tasks required to operate the pool.
	pub(crate) fn start(self: Arc<Self>, token: CancellationToken) {
		let spawn_res = tokio::task::Builder::new()
			.name("PoolsInner::record_metrics")
			.spawn(self.record_metrics_loop(token));
		if let Err(err) = spawn_res {
			tracing::error!(?err, "failed to spawn record_metrics task");
		}
	}
}

impl PoolsInner {
	// MARK: Getters
	pub fn nats_option(&self) -> &Option<NatsPool> {
		&self.nats
	}

	pub fn crdb_map(&self) -> &HashMap<String, CrdbPool> {
		&self.crdb
	}

	pub fn postgres_map(&self) -> &HashMap<String, PostgresPool> {
		&self.postgres
	}

	pub fn cassandra_map(&self) -> &HashMap<String, CassandraPool> {
		&self.cassandra
	}

	pub fn redis_map(&self) -> &HashMap<String, RedisPool> {
		&self.redis
	}

	// MARK: Pool lookups
	pub fn nats(&self) -> Result<NatsPool, Error> {
		self.nats.clone().ok_or(Error::MissingNatsPool)
	}

	pub fn crdb(&self, key: &str) -> Result<CrdbPool, Error> {
		self.crdb
			.get(key)
			.cloned()
			.ok_or_else(|| Error::MissingCrdbPool {
				key: Some(key.to_owned()),
			})
	}

	pub fn postgres(&self, key: &str) -> Result<PostgresPool, Error> {
		self.postgres
			.get(key)
			.cloned()
			.ok_or_else(|| Error::MissingPostgresPool {
				key: Some(key.to_owned()),
			})
	}

	pub fn cassandra(&self, key: &str) -> Result<CassandraPool, Error> {
		self.cassandra
			.get(key)
			.cloned()
			.ok_or_else(|| Error::MissingCassandraPool {
				key: Some(key.to_owned()),
			})
	}

	pub fn redis(&self, key: &str) -> Result<RedisPool, Error> {
		self.redis
			.get(key)
			.cloned()
			.ok_or_else(|| Error::MissingRedisPool {
				key: Some(key.to_owned()),
			})
	}

	pub fn redis_chirp(&self) -> Result<RedisPool, Error> {
		self.redis("redis-chirp")
	}

	pub fn redis_cache(&self) -> Result<RedisPool, Error> {
		self.redis("redis-cache")
	}
}

impl PoolsInner {
	#[tracing::instrument(skip_all)]
	async fn record_metrics_loop(self: Arc<Self>, token: CancellationToken) {
		let cancelled = token.cancelled();
		tokio::pin!(cancelled);

		let mut interval = tokio::time::interval(Duration::from_secs(5));
		loop {
			tokio::select! {
				_ = &mut cancelled => {
					tracing::info!("record metrics cancelled");
					break
				}
				_ = interval.tick() => {
					self.record_metrics().await;
				}
			}
		}
	}

	#[tracing::instrument(skip_all)]
	async fn record_metrics(&self) {
		use crate::metrics::*;

		// CRDB
		for (db_name, pool) in self.crdb_map() {
			let label = &[db_name.as_str()];
			CRDB_POOL_SIZE
				.with_label_values(label)
				.set(pool.size() as i64);
			CRDB_POOL_NUM_IDLE
				.with_label_values(label)
				.set(pool.num_idle() as i64);
		}

		// Postgres
		for (db_name, pool) in self.postgres_map() {
			let label = &[db_name.as_str()];
			POSTGRES_POOL_SIZE
				.with_label_values(label)
				.set(pool.size() as i64);
			POSTGRES_POOL_NUM_IDLE
				.with_label_values(label)
				.set(pool.num_idle() as i64);
		}

		// Cassandra
		for (db_name, pool) in self.cassandra_map() {
			let label = &[db_name.as_str()];
			CASSANDRA_LATENCY_AVERAGE
				.with_label_values(label)
				.set(pool.get_metrics().get_latency_avg_ms().unwrap_or(0) as f64 / 1000.);
			CASSANDRA_LATENCY_P95.with_label_values(label).set(
				pool.get_metrics()
					.get_latency_percentile_ms(95.)
					.unwrap_or(0) as f64 / 1000.,
			);
			CASSANDRA_LATENCY_P99.with_label_values(label).set(
				pool.get_metrics()
					.get_latency_percentile_ms(99.)
					.unwrap_or(0) as f64 / 1000.,
			);
			CASSANDRA_ERRORS_NUM
				.with_label_values(label)
				.set(pool.get_metrics().get_errors_num() as i64);
			CASSANDRA_QUERIES_NUM
				.with_label_values(label)
				.set(pool.get_metrics().get_queries_num() as i64);
			CASSANDRA_ERRORS_ITER_NUM
				.with_label_values(label)
				.set(pool.get_metrics().get_errors_iter_num() as i64);
			CASSANDRA_QUERIES_ITER_NUM
				.with_label_values(label)
				.set(pool.get_metrics().get_queries_iter_num() as i64);
			CASSANDRA_RETRIES_NUM
				.with_label_values(label)
				.set(pool.get_metrics().get_retries_num() as i64);
		}
	}
}
