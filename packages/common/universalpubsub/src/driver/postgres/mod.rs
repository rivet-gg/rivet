use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use anyhow::*;
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64;
use deadpool_postgres::{Config, ManagerConfig, Pool, PoolConfig, RecyclingMethod, Runtime};
use futures_util::future::poll_fn;
use rivet_util::backoff::Backoff;
use tokio::sync::{Mutex, broadcast};
use tokio_postgres::{AsyncMessage, NoTls};
use tracing::Instrument;

use crate::driver::{PubSubDriver, SubscriberDriver, SubscriberDriverHandle};
use crate::pubsub::DriverOutput;

#[derive(Clone)]
struct Subscription {
	// Channel to send messages to this subscription
	tx: broadcast::Sender<Vec<u8>>,
	// Cancellation token shared by all subscribers of this subject
	token: tokio_util::sync::CancellationToken,
}

impl Subscription {
	fn new(tx: broadcast::Sender<Vec<u8>>) -> Self {
		let token = tokio_util::sync::CancellationToken::new();
		Self { tx, token }
	}
}

/// > In the default configuration it must be shorter than 8000 bytes
///
/// https://www.postgresql.org/docs/17/sql-notify.html
const MAX_NOTIFY_LENGTH: usize = 8000;

/// Base64 encoding ratio
const BYTES_PER_BLOCK: usize = 3;
const CHARS_PER_BLOCK: usize = 4;

/// Calculate max message size if encoded as base64
///
/// We need to remove BYTES_PER_BLOCK since there might be a tail on the base64-encoded data that
/// would bump it over the limit.
pub const POSTGRES_MAX_MESSAGE_SIZE: usize =
	(MAX_NOTIFY_LENGTH * BYTES_PER_BLOCK) / CHARS_PER_BLOCK - BYTES_PER_BLOCK;

#[derive(Clone)]
pub struct PostgresDriver {
	pool: Arc<Pool>,
	client: Arc<Mutex<Option<Arc<tokio_postgres::Client>>>>,
	subscriptions: Arc<Mutex<HashMap<String, Subscription>>>,
	client_ready: tokio::sync::watch::Receiver<bool>,
}

impl PostgresDriver {
	#[tracing::instrument(skip(conn_str), fields(memory_optimization))]
	pub async fn connect(conn_str: String, memory_optimization: bool) -> Result<Self> {
		tracing::debug!(?memory_optimization, "connecting to postgres");
		// Create deadpool config from connection string
		let mut config = Config::new();
		config.url = Some(conn_str.clone());
		config.pool = Some(PoolConfig {
			max_size: 64,
			..Default::default()
		});
		config.manager = Some(ManagerConfig {
			recycling_method: RecyclingMethod::Fast,
		});

		// Create the pool
		tracing::debug!("creating postgres pool");
		let pool = config
			.create_pool(Some(Runtime::Tokio1), NoTls)
			.context("failed to create postgres pool")?;
		tracing::debug!("postgres pool created successfully");

		let subscriptions: Arc<Mutex<HashMap<String, Subscription>>> =
			Arc::new(Mutex::new(HashMap::new()));
		let client: Arc<Mutex<Option<Arc<tokio_postgres::Client>>>> = Arc::new(Mutex::new(None));

		// Create channel for client ready notifications
		let (ready_tx, client_ready) = tokio::sync::watch::channel(false);

		// Spawn connection lifecycle task
		tokio::spawn(Self::spawn_connection_lifecycle(
			conn_str.clone(),
			subscriptions.clone(),
			client.clone(),
			ready_tx,
		));

		let driver = Self {
			pool: Arc::new(pool),
			client,
			subscriptions,
			client_ready,
		};

		// Wait for initial connection to be established
		driver.wait_for_client().await?;

		Ok(driver)
	}

	/// Manages the connection lifecycle with automatic reconnection
	async fn spawn_connection_lifecycle(
		conn_str: String,
		subscriptions: Arc<Mutex<HashMap<String, Subscription>>>,
		client: Arc<Mutex<Option<Arc<tokio_postgres::Client>>>>,
		ready_tx: tokio::sync::watch::Sender<bool>,
	) {
		let mut backoff = Backoff::new(8, None, 1_000, 1_000);

		loop {
			match tokio_postgres::connect(&conn_str, tokio_postgres::NoTls).await {
				Result::Ok((new_client, conn)) => {
					tracing::info!("postgres listen connection established");
					// Reset backoff on successful connection
					backoff = Backoff::new(8, None, 1_000, 1_000);

					let new_client = Arc::new(new_client);

					// Update the client reference immediately
					*client.lock().await = Some(new_client.clone());
					// Notify that client is ready
					let _ = ready_tx.send(true);

					// Get channels to re-subscribe to
					let channels: Vec<String> =
						subscriptions.lock().await.keys().cloned().collect();
					let needs_resubscribe = !channels.is_empty();

					if needs_resubscribe {
						tracing::debug!(
							?channels,
							"will re-subscribe to channels after connection starts"
						);
					}

					// Spawn a task to re-subscribe after a short delay
					if needs_resubscribe {
						let client_for_resub = new_client.clone();
						let channels_clone = channels.clone();
						tokio::spawn(async move {
							tracing::debug!(
								?channels_clone,
								"re-subscribing to channels after reconnection"
							);
							for channel in &channels_clone {
								if let Result::Err(e) = client_for_resub
									.execute(&format!("LISTEN \"{}\"", channel), &[])
									.await
								{
									tracing::error!(?e, %channel, "failed to re-subscribe to channel");
								} else {
									tracing::debug!(%channel, "successfully re-subscribed to channel");
								}
							}
						});
					}

					// Poll the connection until it closes
					Self::poll_connection(conn, subscriptions.clone()).await;

					// Clear the client reference on disconnect
					*client.lock().await = None;
					// Notify that client is disconnected
					let _ = ready_tx.send(false);
				}
				Result::Err(e) => {
					tracing::error!(?e, "failed to connect to postgres, retrying");
					backoff.tick().await;
				}
			}
		}
	}

	/// Polls the connection for notifications until it closes or errors
	async fn poll_connection(
		mut conn: tokio_postgres::Connection<
			tokio_postgres::Socket,
			tokio_postgres::tls::NoTlsStream,
		>,
		subscriptions: Arc<Mutex<HashMap<String, Subscription>>>,
	) {
		loop {
			match poll_fn(|cx| conn.poll_message(cx)).await {
				Some(std::result::Result::Ok(AsyncMessage::Notification(note))) => {
					tracing::trace!(channel = %note.channel(), "received notification");
					if let Some(sub) = subscriptions.lock().await.get(note.channel()).cloned() {
						let bytes = match BASE64.decode(note.payload()) {
							std::result::Result::Ok(b) => b,
							std::result::Result::Err(err) => {
								tracing::error!(?err, "failed decoding base64");
								continue;
							}
						};
						tracing::trace!(channel = %note.channel(), bytes_len = bytes.len(), "sending to broadcast channel");
						let _ = sub.tx.send(bytes);
					} else {
						tracing::warn!(channel = %note.channel(), "received notification for unknown channel");
					}
				}
				Some(std::result::Result::Ok(_)) => {
					// Ignore other async messages
				}
				Some(std::result::Result::Err(err)) => {
					tracing::error!(?err, "postgres connection error, reconnecting");
					break; // Exit loop to reconnect
				}
				None => {
					tracing::warn!("postgres connection closed, reconnecting");
					break; // Exit loop to reconnect
				}
			}
		}
	}

	/// Wait for the client to be connected
	async fn wait_for_client(&self) -> Result<Arc<tokio_postgres::Client>> {
		let mut ready_rx = self.client_ready.clone();
		tokio::time::timeout(tokio::time::Duration::from_secs(5), async {
			loop {
				// Subscribe to changed before attempting to access client
				let changed_fut = ready_rx.changed();

				// Check if client is already available
				if let Some(client) = self.client.lock().await.clone() {
					return Ok(client);
				}

				// Wait for change, will return client if exists on next iteration
				changed_fut
					.await
					.map_err(|_| anyhow!("connection lifecycle task ended"))?;
				tracing::debug!("client does not exist immediately after receive ready");
			}
		})
		.await
		.map_err(|_| anyhow!("timeout waiting for postgres client connection"))?
	}

	fn hash_subject(&self, subject: &str) -> String {
		// Postgres channel names have a 64 character limit
		// Hash the subject to ensure it fits
		let mut hasher = DefaultHasher::new();
		subject.hash(&mut hasher);
		format!("ups_{:x}", hasher.finish())
	}
}

#[async_trait]
impl PubSubDriver for PostgresDriver {
	async fn subscribe(&self, subject: &str) -> Result<SubscriberDriverHandle> {
		// TODO: To match NATS implementation, LISTEN must be pipelined (i.e. wait for the command
		// to reach the server, but not wait for it to respond). However, this has to ensure that
		// NOTIFY & LISTEN are called on the same connection (not diff connections in a pool) or
		// else there will be race conditions where messages might be published before
		// subscriptions are registered.
		//
		// tokio-postgres currently does not expose the API for pipelining, so we are SOL.
		//
		// We might be able to use a background tokio task in combination with flush if we use the
		// same Postgres connection, but unsure if that will create a bottleneck.

		let hashed = self.hash_subject(subject);

		// Check if we already have a subscription for this channel
		let (rx, drop_guard) =
			if let Some(existing_sub) = self.subscriptions.lock().await.get(&hashed).cloned() {
				// Reuse the existing broadcast channel
				let rx = existing_sub.tx.subscribe();
				let drop_guard = existing_sub.token.clone().drop_guard();
				(rx, drop_guard)
			} else {
				// Create a new broadcast channel for this subject
				let (tx, rx) = tokio::sync::broadcast::channel(1024);
				let subscription = Subscription::new(tx.clone());

				// Register subscription
				self.subscriptions
					.lock()
					.await
					.insert(hashed.clone(), subscription.clone());

				// Execute LISTEN command on the async client (for receiving notifications)
				// This only needs to be done once per channel
				// Wait for client to be connected with retry logic
				let client = self.wait_for_client().await?;
				let span = tracing::trace_span!("pg_listen");
				client
					.execute(&format!("LISTEN \"{hashed}\""), &[])
					.instrument(span)
					.await?;

				// Spawn a single cleanup task for this subscription waiting on its token
				let driver = self.clone();
				let hashed_clone = hashed.clone();
				let tx_clone = tx.clone();
				let token_clone = subscription.token.clone();
				tokio::spawn(async move {
					token_clone.cancelled().await;
					if tx_clone.receiver_count() == 0 {
						let client = driver.client.lock().await.clone();
						if let Some(client) = client {
							let sql = format!("UNLISTEN \"{}\"", hashed_clone);
							if let Err(err) = client.execute(sql.as_str(), &[]).await {
								tracing::warn!(?err, %hashed_clone, "failed to UNLISTEN channel");
							} else {
								tracing::trace!(%hashed_clone, "unlistened channel");
							}
						}
						driver.subscriptions.lock().await.remove(&hashed_clone);
					}
				});

				let drop_guard = subscription.token.clone().drop_guard();
				(rx, drop_guard)
			};

		Ok(Box::new(PostgresSubscriber {
			subject: subject.to_string(),
			rx: Some(rx),
			_drop_guard: drop_guard,
		}))
	}

	async fn publish(&self, subject: &str, payload: &[u8]) -> Result<()> {
		// TODO: See `subscribe` about pipelining

		// Encode payload to base64 and send NOTIFY
		let encoded = BASE64.encode(payload);
		let conn = self.pool.get().await?;
		let hashed = self.hash_subject(subject);
		let span = tracing::trace_span!("pg_notify");
		conn.execute(&format!("NOTIFY \"{hashed}\", '{encoded}'"), &[])
			.instrument(span)
			.await?;

		Ok(())
	}

	async fn flush(&self) -> Result<()> {
		Ok(())
	}

	fn max_message_size(&self) -> usize {
		POSTGRES_MAX_MESSAGE_SIZE
	}
}

pub struct PostgresSubscriber {
	subject: String,
	rx: Option<tokio::sync::broadcast::Receiver<Vec<u8>>>,
	_drop_guard: tokio_util::sync::DropGuard,
}

#[async_trait]
impl SubscriberDriver for PostgresSubscriber {
	async fn next(&mut self) -> Result<DriverOutput> {
		let rx = match self.rx.as_mut() {
			Some(rx) => rx,
			None => return Ok(DriverOutput::Unsubscribed),
		};
		match rx.recv().await {
			std::result::Result::Ok(payload) => Ok(DriverOutput::Message {
				subject: self.subject.clone(),
				payload,
			}),
			Err(tokio::sync::broadcast::error::RecvError::Closed) => Ok(DriverOutput::Unsubscribed),
			Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
				// Try again
				self.next().await
			}
		}
	}
}
