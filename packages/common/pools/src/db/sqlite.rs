use std::{
	collections::HashMap,
	ops::Deref,
	sync::{Arc, Weak},
	time::Duration,
};
use tokio::sync::broadcast;
use sqlx::{
	migrate::MigrateDatabase,
	sqlite::{
		SqliteAutoVacuum, SqliteConnectOptions, SqliteConnection, SqlitePoolOptions,
		SqliteSynchronous,
	},
	Executor, Sqlite,
};
use tokio::{sync::Mutex, time::Instant};
use global_error::{GlobalError, GlobalResult, unwrap};
use foundationdb::{self, directory::{DirectoryLayer, Directory}};
use crate::{Error, FdbPool};

const GC_INTERVAL: Duration = Duration::from_secs(1);
const POOL_TTL: Duration = Duration::from_secs(15);
const SQLITE_SNAPSHOT_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes
const CHUNK_SIZE: usize = 9 * 1024; // 9KB to stay safely under FDB's 10KB limit

/// Pool wrapper that tracks when it was dropped to automatically update the last used on
/// `SqlitePoolEntry` in order to know when to GC the pool.
pub struct SqlitePool {
    /// Reference to the inner pool. Holds a reference as a separate Arc so we can hold a weak
    /// reference in `SqlitePoolEntry` to know if the pool is dropped.
	pool: Arc<sqlx::SqlitePool>,
}

impl Drop for SqlitePoolManager {
    fn drop(&mut self) {
        // Ignore send errors - receivers may already be dropped
        let _ = self.shutdown.send(());
    }
}

impl Deref for SqlitePool {
	type Target = sqlx::SqlitePool;

	fn deref(&self) -> &Self::Target {
		&*self.pool
	}
}

struct SqlitePoolEntry {
	/// Weak reference to the pool. Used to determine if we can safely drop this pool if all other
	/// references have been dropped.
	pool: Weak<sqlx::SqlitePool>,

	/// Last time this pool was accessed (either by `get` or a ref was dropped, meaning the query
	/// ended)
	last_access: Instant,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct Key {
	key: String,
	read_only: bool,
}

#[derive(Clone)]
pub struct SqlitePoolManager {
    pools: Arc<Mutex<HashMap<Key, SqlitePoolEntry>>>,
    shutdown: broadcast::Sender<()>,
    fdb: Option<FdbPool>,
    force_local: bool,
}

impl SqlitePoolManager {
    pub fn new(fdb: Option<FdbPool>) -> Self {
        let pools = Arc::new(Mutex::new(HashMap::new()));
        let (shutdown, _) = broadcast::channel(1);
        let shutdown_rx = shutdown.subscribe();
        let force_local = std::env::var("_RIVET_POOL_SQLITE_FORCE_LOCAL")
            .map(|x| x == "1")
            .unwrap_or(false);
        
        let pools_clone = pools.clone();
        let manager = SqlitePoolManager { 
            pools: pools_clone,
            shutdown,
            fdb,
            force_local,
        };

        tokio::task::spawn(Self::start(pools.clone(), shutdown_rx));
        
        manager
    }

    async fn read_from_fdb(&self, key: &str) -> GlobalResult<Option<Vec<u8>>> {
        let fdb = unwrap!(self.fdb.as_ref());
        let data = fdb.run(|tx, _mc| {
            let key = key.to_string();
            async move {
                let mut chunks = Vec::new();
                let subspace = DirectoryLayer::default()
                    .create_or_open(&tx, &["rivet", "sqlite", &key], None, None)
                    .await?;

                // Read all chunks
                let range = subspace.range();
                let kvs = tx.get_ranges_keyvalues_owned(
                    fdb::RangeOption {
                        mode: fdb::options::StreamingMode::WantAll,
                        ..subspace.range().into()
                    },
                    false,
                ).await?;
                for kv in kvs {
                    let (_, idx) = subspace.unpack::<(String, usize)>(kv.key())?;
                    chunks.push((idx, kv.value().to_vec()));
                }

                // Sort and combine chunks
                chunks.sort_by_key(|(idx, _)| *idx);
                let data = chunks.into_iter()
                    .flat_map(|(_, chunk)| chunk)
                    .collect::<Vec<_>>();

                Ok::<_, GlobalError>(if data.is_empty() { None } else { Some(data) }).map_err(|e| GlobalError::Internal(e.into()))?
            }
        }).await?;

        Ok(data)
    }

    async fn write_to_fdb(&self, key: &str, data: &[u8]) -> GlobalResult<()> {
        let fdb = unwrap!(self.fdb.as_ref());
        fdb.run(|tx, _mc| {
            let key = key.to_string();
            let data = data.to_vec();
            async move {
                let subspace = DirectoryLayer::default()
                    .create_or_open(&tx, &["rivet", "sqlite", &key], None, None)
                    .await?;

                // Clear previous data
                let range = subspace.range();
                tx.clear_range(range.0, range.1);

                // Write chunks
                for (idx, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
                    tx.set(
                        &subspace.pack(&("data", idx)),
                        chunk
                    );
                }

                Ok(())
            }
        }).await
    }

	async fn start(pools: Arc<Mutex<HashMap<Key, SqlitePoolEntry>>>, mut shutdown: broadcast::Receiver<()>) {
		let mut interval = tokio::time::interval(GC_INTERVAL);

		loop {
			tokio::select! {
				_ = interval.tick() => {},
				Ok(_) = shutdown.recv() => {
					tracing::debug!("shutting down sqlite pool manager");
					break;
				}
			}

			// Anything last used before this instant will be removed
			let expire_ts = Instant::now() - POOL_TTL;

			// Remove pools
			{
				let mut pools_guard = pools.lock().await;
				let mut removed = 0;
				pools_guard.retain(|k, v| {
					if v.last_access > expire_ts {
                        let ref_count = v.pool.strong_count();
                        if ref_count == 0 {
                            true
                        } else {
                            tracing::warn!(?ref_count, ?k, "sqlite pool is expired and should have no references, but still has references");
                            false
                        }
					} else {
						removed += 1;
						false
					}
				});

                tracing::debug!(?removed, "gced sqlite pools");
			}
		}
	}

	/// Get or creates an sqlite pool for the given key
	///
	/// IMPORTANT: Do not hold a reference to this value for an extended period of time. We use
	/// this function call to determine when to GC a pool.
	pub async fn get(&self, key: &str, read_only: bool) -> Result<SqlitePool, Error> {
		let mut pools_guard = self.pools.lock().await;

		let key = Key {
			key: key.to_string(),
			read_only,
		};

		if let Some(entry) = pools_guard.get_mut(&key) {
			if let Some(pool) = entry.pool.upgrade() {
				entry.last_access = Instant::now();
				return Ok(SqlitePool { pool });
			}
		};

        // Load from FDB if enabled
        let db_path = if !self.force_local && self.fdb.is_some() {
            let temp_dir = std::env::temp_dir();
            let db_path = temp_dir.join(format!("rivet-sqlite-{}.db", key.key));
            
            if let Some(data) = self.read_from_fdb(&key.key).await.map_err(Error::Fdb)? {
                tokio::fs::write(&db_path, data).await.map_err(Error::Io)?;
            }
            
            db_path.to_str().unwrap().to_string()
        } else {
            format!("/var/lib/rivet-sqlite/{}.db", key.key)
        };

        let db_url = format!("sqlite://{}", db_path);

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
			.read_only(read_only)
			// Set synchronous mode to NORMAL for performance and data safety balance
			.synchronous(SqliteSynchronous::Normal)
			// Set busy timeout to 5 seconds to avoid "database is locked" errors
			.busy_timeout(Duration::from_secs(5))
			// Enable foreign key constraint enforcement
			.foreign_keys(true)
			// Enable auto vacuuming and set it to incremental mode for gradual space reclaiming
			.auto_vacuum(SqliteAutoVacuum::Incremental);

		let pool_raw = SqlitePoolOptions::new()
			.max_lifetime_jitter(Duration::from_secs(90))
			// Open connection immediately on startup
			.min_connections(1)
			.after_connect(|conn, _meta| Box::pin(async move { setup_pragma(conn).await }))
			.connect_with(opts)
			.await
			.map_err(Error::BuildSqlx)?;
        let pool = Arc::new(pool_raw);

        tracing::debug!(?key, "sqlite connected");

        if !self.force_local && self.fdb.is_some() {
            let key = key.clone();
            let db_path = db_path.clone();
            let this = self.clone();
            
            tokio::task::spawn(async move {
                let mut interval = tokio::time::interval(SQLITE_SNAPSHOT_INTERVAL);
                loop {
                    interval.tick().await;
                    
                    // Read current DB file
                    if let Ok(data) = tokio::fs::read(&db_path).await {
                        if let Err(err) = this.write_to_fdb(&key.key, &data).await {
                            tracing::error!(?err, "failed to snapshot sqlite db to fdb");
                        }
                    }
                }
            });
        }

        pools_guard.insert(
            key,
            SqlitePoolEntry {
                pool: Arc::downgrade(&pool),
                last_access: Instant::now(),
            },
        );

        Ok(SqlitePool { pool })
	}
}

async fn setup_pragma(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
	let settings = [
		// NOTE: sqlx doesnt seem to have a WAL2 option so we set it with a PRAGMA query
		// Set the journal mode to Write-Ahead Logging 2 for concurrency
		"PRAGMA journal_mode = WAL2",
	];

	conn.execute(settings.join("; ").as_str()).await?;

	Ok(())
}
