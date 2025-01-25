use std::{collections::HashMap, sync::Arc, time::Duration};

use futures_util::{StreamExt, TryStreamExt};
use sqlite_util::SqlitePoolExt;
use sqlx::{
	migrate::MigrateDatabase,
	sqlite::{SqliteConnectOptions, SqlitePoolOptions},
	Sqlite,
};
use tokio::sync::Mutex;

use crate::Error;

pub type SqlitePool = sqlx::SqlitePool;

#[derive(Debug, Hash, Eq, PartialEq)]
struct Key {
	key: String,
	read_only: bool,
}

#[derive(Clone)]
pub struct SqlitePoolManager {
	// TODO: Somehow remove old pools
	pools: Arc<Mutex<HashMap<Key, SqlitePool>>>,
}

impl SqlitePoolManager {
	pub fn new() -> Self {
		SqlitePoolManager {
			pools: Arc::new(Mutex::new(HashMap::new())),
		}
	}

	/// Get or creates an sqlite pool for the given key
	pub async fn get(&self, key: &str, read_only: bool) -> Result<SqlitePool, Error> {
		let mut pools_guard = self.pools.lock().await;

		let key = Key {
			key: key.to_string(),
			read_only,
		};

		let pool = if let Some(pool) = pools_guard.get(&key) {
			pool.clone()
		} else {
			// TODO: Hardcoded for testing
			let db_url = format!("sqlite:///var/lib/rivet-sqlite/{}.db", key.key);

			tracing::debug!(?key, "sqlite connecting");

			// Init if doesn't exist
			if !Sqlite::database_exists(&db_url)
				.await
				.map_err(Error::BuildSqlx)?
			{
				Sqlite::create_database(&db_url)
					.await
					.map_err(Error::BuildSqlx)?;
			}

			let opts = db_url
				.parse::<SqliteConnectOptions>()
				.map_err(Error::BuildSqlx)?
				.read_only(read_only);

			let pool = SqlitePoolOptions::new()
				.max_lifetime_jitter(Duration::from_secs(90))
				// Open connection immediately on startup
				.min_connections(1)
				.connect_with(opts)
				.await
				.map_err(Error::BuildSqlx)?;

			// Run at the start of every connection
			setup_pragma(&pool).await.map_err(Error::BuildSqlx)?;

			tracing::debug!(?key, "sqlite connected");

			pools_guard.insert(key, pool.clone());

			pool
		};

		Ok(pool)
	}
}

async fn setup_pragma(pool: &SqlitePool) -> Result<(), sqlx::Error> {
	// Has to be String instead of static str due to a weird compiler bug. This crate will compile just fine
	// but chirp-workflow will not and the error has nothing to do with this code
	let settings = [
		// Set the journal mode to Write-Ahead Logging 2 for concurrency
		"PRAGMA journal_mode = WAL2".to_string(),
		// Set synchronous mode to NORMAL for performance and data safety balance
		"PRAGMA synchronous = NORMAL".to_string(),
		// Set busy timeout to 5 seconds to avoid "database is locked" errors
		"PRAGMA busy_timeout = 5000".to_string(),
		// Enable foreign key constraint enforcement
		"PRAGMA foreign_keys = ON".to_string(),
		// Enable auto vacuuming and set it to incremental mode for gradual space reclaiming
		"PRAGMA auto_vacuum = INCREMENTAL".to_string(),
	];

	futures_util::stream::iter(settings)
		.map(|setting| {
			let pool = pool.clone();
			async move {
				let mut conn = pool.conn().await?;
				sqlx::query(&setting).execute(&mut *conn).await
			}
		})
		.buffer_unordered(16)
		.try_collect::<Vec<_>>()
		.await?;

	Ok(())
}
