//! Actor-owned JSON input stream. DS owns wake retries; core owns processing and
//! the checkpoint. Successful handler side effects can repeat after a crash.
use super::stream_subscription::NamedStreamSubscription;
use rivetkit_actor_persist::{
	STREAM_CHECKPOINT_VERSION, generated::stream_checkpoint_v2 as wire,
	stream_checkpoint::StreamCheckpoint,
};
use sha2::{Digest, Sha256};
use std::{
	collections::{BTreeMap, HashSet},
	time::{Duration, SystemTime, UNIX_EPOCH},
};
use vbare::OwnedVersionedData;

use anyhow::{Context, Result, ensure};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, VerifyingKey};
use reqwest::{Client, Method};
use rivet_error::RivetError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use super::{
	ActorContext, ActorEvent, RequestSaveOpts, config::InputStreamConfig, internal_storage,
};

mod dead_letter;

pub const WAKE_PATH: &str = "/internal/streams/wake";
pub const APPEND_PATH: &str = "/internal/streams/append";
// A single fixed core-owned checkpoint in the existing indexed metadata table.
const CHECKPOINT_KEY: &str = "input_stream_checkpoint_v1";
const MAX_HANDLER_FAILURES: u32 = 6;
const MAX_RESPONSE: usize = 16 * 1024 * 1024;

#[derive(Debug, RivetError, Serialize)]
#[error("stream")]
pub enum StreamError {
	#[error("not_configured", "Actor input stream is not configured.")]
	NotConfigured,
	#[error("invalid_wake", "Invalid stream wake notification.")]
	InvalidWake,
	#[error("busy", "Actor is already processing a stream wake.")]
	Busy,
	#[error("not_paused", "Stream source is not paused.")]
	NotPaused,
}

#[derive(Serialize, Deserialize)]
struct Checkpoint {
	offset: String,
	completed: usize,
	#[serde(default)]
	failure: Option<wire::Failure>,
	#[serde(default)]
	dead_letter: Option<wire::DeadLetter>,
}

fn checkpoint_key(handler: &str, path: &str) -> Result<String> {
	Ok(if handler == "primary" {
		CHECKPOINT_KEY.to_owned()
	} else {
		let digest = Sha256::digest(serde_json::to_vec(&(handler, path))?);
		format!("stream_checkpoint_v1_{digest:x}")
	})
}
fn now_ms() -> i64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap_or_default()
		.as_millis() as i64
}
fn retry_delay(attempts: u32) -> i64 {
	1_000_i64.saturating_mul(1_i64 << attempts.saturating_sub(1).min(5))
}
impl Checkpoint {
	fn decode(raw: &str) -> Result<Self> {
		// Read the original JSON checkpoint once, then migrate on the next write.
		let Some(encoded) = raw.strip_prefix("bare:") else {
			return Ok(serde_json::from_str(raw)?);
		};
		let data =
			StreamCheckpoint::deserialize_with_embedded_version(&URL_SAFE_NO_PAD.decode(encoded)?)?;
		Ok(Self {
			offset: data.offset,
			completed: usize::try_from(data.completed)?,
			failure: data.failure,
			dead_letter: data.dead_letter,
		})
	}
	fn encode(&self) -> Result<String> {
		let bytes = StreamCheckpoint::wrap_latest(wire::Data {
			offset: self.offset.clone(),
			completed: self.completed as u64,
			failure: self.failure.clone(),
			dead_letter: self.dead_letter.clone(),
		})
		.serialize_with_embedded_version(STREAM_CHECKPOINT_VERSION)?;
		Ok(format!("bare:{}", URL_SAFE_NO_PAD.encode(bytes)))
	}
	async fn save(&self, ctx: &ActorContext, key: &str) -> Result<()> {
		internal_storage::persist_meta_text(ctx.sql(), key, &self.encode()?).await
	}
	fn failed(&mut self, error: &anyhow::Error, now: i64) {
		let previous = self.failure.take();
		let attempts = previous
			.as_ref()
			.map_or(1, |f| f.attempts.saturating_add(1));
		let error = RivetError::extract(error);
		let error = serde_json::to_string(&json!({
			"group": error.group(),
			"code": error.code(),
			"message": error.message().chars().take(4096).collect::<String>(),
		}))
		.expect("error object");
		self.failure = Some(wire::Failure {
			attempts,
			first_failed_at: previous.map_or(now, |f| f.first_failed_at),
			last_failed_at: now,
			next_attempt_at: now.saturating_add(retry_delay(attempts)),
			paused: attempts >= MAX_HANDLER_FAILURES,
			notified: false,
			error,
		});
	}
	fn failure_info(&self, handler: &str, path: &str) -> Value {
		let Some(f) = &self.failure else {
			return Value::Null;
		};
		json!({"subscription":handler,"stream":path,"offset":self.offset,"messageIndex":self.completed,
            "attempts":f.attempts,"firstFailedAt":f.first_failed_at,"lastFailedAt":f.last_failed_at,
            "nextAttemptAt":if f.paused {None} else {Some(f.next_attempt_at)},"paused":f.paused,
            "error":serde_json::from_str::<Value>(&f.error).unwrap_or(Value::Null)})
	}
}

/// Select a single subscribed source. Primary input requires no arguments.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StreamSource {
	pub subscription: Option<String>,
	pub stream: Option<String>,
}

#[derive(Deserialize)]
struct Wake {
	#[serde(skip)]
	handler: String,
	subscription_id: String,
	wake_id: String,
	generation: i64,
	streams: Vec<WakeStream>,
	callback_url: String,
	callback_token: String,
}

#[derive(Deserialize)]
struct WakeStream {
	path: String,
	acked_offset: String,
	tail_offset: String,
}

struct Connection {
	dead_letters: BTreeMap<String, super::config::DeadLetterConfig>,
	primary: bool,
	actor_key: crate::types::ActorKey,
	subscriptions: BTreeMap<String, NamedStreamSubscription>,
	client: Client,
	root: url::Url,
	token: Option<String>,
	path: String,
	subscription: String,
}

impl Connection {
	fn new(ctx: &ActorContext, config: InputStreamConfig) -> Result<Self> {
		let root = url::Url::parse(&config.endpoint)?;
		ensure!(
			matches!(root.scheme(), "http" | "https")
				&& root.host_str().is_some()
				&& root.username().is_empty()
				&& root.password().is_none()
				&& root.query().is_none()
				&& root.fragment().is_none(),
			"Invalid DS endpoint"
		);
		let mut primary_url = root.clone();
		primary_url
			.path_segments_mut()
			.map_err(|_| StreamError::NotConfigured.build())?
			.clear()
			.push("actors")
			.push(ctx.name())
			.push(ctx.actor_id());
		let primary_path = primary_url.path().trim_start_matches('/').to_owned();
		Ok(Self {
			primary: config.primary,
			dead_letters: config.dead_letters,
			actor_key: ctx.key().clone(),
			subscriptions: config.subscriptions,
			client: Client::builder()
				.redirect(reqwest::redirect::Policy::none())
				.timeout(Duration::from_secs(10))
				.build()?,
			root,
			token: config
				.token
				.or_else(|| ctx.client_token().map(str::to_owned)),
			path: primary_path,
			subscription: format!("actor-{}", ctx.actor_id()),
		})
	}

	fn url(&self, path: &str) -> Result<url::Url> {
		let mut url = self.root.clone();
		let mut segments = url
			.path_segments_mut()
			.map_err(|_| StreamError::NotConfigured.build())?;
		segments.pop_if_empty();
		for segment in path.split('/') {
			segments.push(segment);
		}
		drop(segments);
		Ok(url)
	}

	fn request(&self, method: Method, path: &str) -> Result<reqwest::RequestBuilder> {
		let mut request = self.client.request(method, self.url(path)?);
		if let Some(token) = &self.token {
			request = request.bearer_auth(token);
		}
		Ok(request)
	}

	fn source_request(&self, method: Method, path: &str) -> Result<reqwest::RequestBuilder> {
		self.source_request_with_token(method, path, self.token.as_deref())
	}

	fn source_request_with_token(
		&self,
		method: Method,
		path: &str,
		token: Option<&str>,
	) -> Result<reqwest::RequestBuilder> {
		ensure!(
			!path.starts_with('/') && !path.contains(['?', '#', '\\']) && path.len() <= 2048,
			StreamError::InvalidWake.build()
		);
		let expected = format!("{}/{}", self.root.path().trim_end_matches('/'), path);
		let mut url = self.root.clone();
		url.set_path(&expected);
		ensure!(url.path() == expected, StreamError::InvalidWake.build());
		let mut request = self.client.request(method, url);
		if let Some(token) = token {
			request = request.bearer_auth(token);
		}
		Ok(request)
	}

	async fn callback(&self, wake: &Wake, done: bool) -> Result<()> {
		self.callback_progress(
			wake,
			done,
			if done {
				wake.streams.iter().collect()
			} else {
				vec![]
			},
		)
		.await
	}
	async fn callback_progress(
		&self,
		wake: &Wake,
		done: bool,
		streams: Vec<&WakeStream>,
	) -> Result<()> {
		// Never follow an arbitrary callback URL from the request body.
		let url = self.url(&format!(
			"__ds/subscriptions/{}/callback",
			wake.subscription_id
		))?;
		let mut body = json!({"wake_id":wake.wake_id,"generation":wake.generation,"done":done});
		if !streams.is_empty() {
			body["acks"] = json!(
				streams
					.into_iter()
					.map(|s| json!({"stream":s.path,"offset":s.tail_offset}))
					.collect::<Vec<_>>()
			);
		}
		checked(
			self.client
				.post(url)
				.bearer_auth(&wake.callback_token)
				.json(&body),
		)
		.await?;
		Ok(())
	}

	async fn verify(&self, signature: &str, body: &[u8]) -> Result<Wake> {
		ensure!(body.len() <= 64 * 1024, StreamError::InvalidWake.build());
		let fields = signature
			.split(',')
			.map(|field| field.trim().split_once('='))
			.collect::<Option<Vec<_>>>()
			.ok_or_else(|| StreamError::InvalidWake.build())?;
		ensure!(fields.len() == 3, StreamError::InvalidWake.build());
		let field = |name| -> Result<&str> {
			let values = fields
				.iter()
				.filter(|(key, _)| *key == name)
				.collect::<Vec<_>>();
			ensure!(values.len() == 1, StreamError::InvalidWake.build());
			Ok(values[0].1)
		};
		let timestamp = field("t")?;
		let ts: u64 = timestamp
			.parse()
			.map_err(|_| StreamError::InvalidWake.build())?;
		let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
		ensure!(now.abs_diff(ts) <= 300, StreamError::InvalidWake.build());
		let kid = field("kid")?;
		let signature = Signature::from_slice(
			&URL_SAFE_NO_PAD
				.decode(field("ed25519")?)
				.map_err(|_| StreamError::InvalidWake.build())?,
		)
		.map_err(|_| StreamError::InvalidWake.build())?;
		let (_, bytes) = checked(self.request(Method::GET, "__ds/jwks.json")?).await?;
		let jwks: Value = serde_json::from_slice(&bytes)?;
		let key = jwks["keys"]
			.as_array()
			.and_then(|keys| {
				keys.iter()
					.find(|key| key["kid"] == kid && key["kty"] == "OKP" && key["crv"] == "Ed25519")
			})
			.ok_or_else(|| StreamError::InvalidWake.build())?;
		let public: [u8; 32] = URL_SAFE_NO_PAD
			.decode(
				key["x"]
					.as_str()
					.ok_or_else(|| StreamError::InvalidWake.build())?,
			)?
			.try_into()
			.map_err(|_| StreamError::InvalidWake.build())?;
		let mut signed = format!("{timestamp}.").into_bytes();
		signed.extend_from_slice(body);
		VerifyingKey::from_bytes(&public)?
			.verify_strict(&signed, &signature)
			.map_err(|_| StreamError::InvalidWake.build())?;
		let mut wake: Wake =
			serde_json::from_slice(body).map_err(|_| StreamError::InvalidWake.build())?;

		wake.handler = self.validate_destination(&wake)?;
		ensure!(
			!wake.wake_id.is_empty()
				&& wake.generation >= 0
				&& wake.callback_url
					== self
						.url(&format!(
							"__ds/subscriptions/{}/callback",
							wake.subscription_id
						))?
						.as_str(),
			StreamError::InvalidWake.build()
		);
		Ok(wake)
	}
	fn validate_destination(&self, wake: &Wake) -> Result<String> {
		ensure!(
			!wake.streams.is_empty() && wake.streams.len() <= 4096,
			StreamError::InvalidWake.build()
		);
		let mut seen = HashSet::new();
		ensure!(
			wake.streams.iter().all(|s| seen.insert(&s.path)),
			StreamError::InvalidWake.build()
		);
		if wake.subscription_id == self.subscription {
			ensure!(
				self.primary && wake.streams.len() == 1 && wake.streams[0].path == self.path,
				StreamError::InvalidWake.build()
			);
			return Ok("primary".into());
		}
		let name = wake
			.subscription_id
			.strip_prefix(&format!("{}-", self.subscription))
			.ok_or_else(|| StreamError::InvalidWake.build())?;
		let route = self
			.subscriptions
			.get(name)
			.ok_or_else(|| StreamError::InvalidWake.build())?;
		for stream in &wake.streams {
			let resolved = route
				.resolve_key(&stream.path)?
				.ok_or_else(|| StreamError::InvalidWake.build())?;
			let expected = resolved
				.into_iter()
				.map(crate::types::ActorKeySegment::String)
				.collect::<Vec<_>>();
			ensure!(expected == self.actor_key, StreamError::InvalidWake.build());
		}
		Ok(name.to_owned())
	}
}

#[cfg(test)]
#[path = "../../tests/unit/stream.rs"]
mod tests;

async fn checked(
	request: reqwest::RequestBuilder,
) -> Result<(reqwest::header::HeaderMap, Vec<u8>)> {
	let mut response = request.send().await.map_err(reqwest::Error::without_url)?;
	if !response.status().is_success() {
		tracing::warn!(status = %response.status(), "Durable Streams request rejected");
	}
	ensure!(
		response.status().is_success(),
		"DS HTTP {}",
		response.status()
	);
	let headers = response.headers().clone();
	let mut body = Vec::new();
	while let Some(chunk) = response
		.chunk()
		.await
		.map_err(reqwest::Error::without_url)?
	{
		ensure!(
			body.len() + chunk.len() <= MAX_RESPONSE,
			"DS response exceeds input limit"
		);
		body.extend_from_slice(&chunk);
	}
	Ok((headers, body))
}

impl ActorContext {
	pub async fn stream_append_cbor(&self, body: &[u8]) -> Result<()> {
		self.stream_append(ciborium::from_reader(body)?).await
	}
	fn stream_connection(&self) -> Result<Connection> {
		let config = self
			.0
			.queue_config
			.lock()
			.input_stream
			.clone()
			.ok_or_else(|| StreamError::NotConfigured.build())?;
		Connection::new(self, config)
	}

	/// Durable acceptance, not processing completion. A JSON value is one record;
	/// the singleton outer array preserves array-valued messages.
	pub async fn stream_append(&self, body: Value) -> Result<()> {
		let token = self.stream_connection()?.token.clone();
		self.stream_append_authenticated(body, token.as_deref())
			.await
	}

	/// Public append must retain the caller's authority through the DS boundary.
	pub async fn stream_append_authenticated(
		&self,
		body: Value,
		token: Option<&str>,
	) -> Result<()> {
		let token = token
			.filter(|t| !t.is_empty())
			.ok_or_else(|| StreamError::InvalidWake.build())?;
		let _awake = self.keep_awake_region();
		let connection = self.stream_connection()?;
		ensure!(connection.primary, StreamError::NotConfigured.build());
		self.0
			.stream_initialize
			.get_or_try_init(|| async {
				checked(
					connection
						.source_request_with_token(Method::PUT, &connection.path, Some(token))?
						.header("content-type", "application/json"),
				)
				.await?;

				Ok::<_, anyhow::Error>(())
			})
			.await?;
		checked(
			connection
				.source_request_with_token(Method::POST, &connection.path, Some(token))?
				.json(&json!([body])),
		)
		.await?;
		Ok(())
	}

	/// Verify before accepting work. DS will retry a rejected/busy notification.
	pub async fn stream_wake(&self, signature: &str, body: &[u8]) -> Result<()> {
		let awake = self.keep_awake_region();
		let connection = self.stream_connection()?;
		let wake = connection.verify(signature, body).await?;
		let lock = self
			.0
			.stream_lock
			.clone()
			.try_lock_owned()
			.map_err(|_| StreamError::Busy.build())?;
		let ctx = self.clone();
		self.register_task(async move {
			let _awake = awake;
			let _lock = lock;
			let cancel = CancellationToken::new();
			let shutdown = ctx.shutdown_deadline_token();
			let result = tokio::select! {
				result = ctx.drain_stream(&connection, &wake, &cancel) => result,
				result = async {
					loop {
						tokio::time::sleep(Duration::from_secs(10)).await;
						connection.callback(&wake, false).await?;
					}
					#[allow(unreachable_code)] Ok::<(), anyhow::Error>(())
				} => result,
				_ = shutdown.cancelled() => Err(anyhow::anyhow!("Stream processing canceled during shutdown")),
			};
			cancel.cancel();
			if let Err(error) = result {
				tracing::error!(
					actor_id = ctx.actor_id(),
					?error,
					"actor input stream processing failed; delivery remains unacknowledged"
				);
			}
		});
		Ok(())
	}

	async fn drain_stream(
		&self,
		connection: &Connection,
		wake: &Wake,
		cancel: &CancellationToken,
	) -> Result<()> {
		let mut completed = Vec::new();
		for stream in &wake.streams {
			if self
				.drain_source(connection, &wake.handler, stream, cancel)
				.await?
			{
				completed.push(stream);
			}
		}
		connection
			.callback_progress(wake, completed.len() == wake.streams.len(), completed)
			.await
	}

	fn stream_source(
		&self,
		connection: &Connection,
		source: StreamSource,
	) -> Result<(String, String)> {
		let handler = source.subscription.unwrap_or_else(|| "primary".into());
		let path = if handler == "primary" {
			ensure!(
				connection.primary && source.stream.as_ref().is_none_or(|p| p == &connection.path),
				StreamError::NotConfigured.build()
			);
			connection.path.clone()
		} else {
			let path = source
				.stream
				.context("Named subscriptions require a stream path")?;
			let route = connection
				.subscriptions
				.get(&handler)
				.ok_or_else(|| StreamError::NotConfigured.build())?;
			let key = route
				.resolve_key(&path)?
				.ok_or_else(|| StreamError::NotConfigured.build())?;
			ensure!(
				key.into_iter()
					.map(crate::types::ActorKeySegment::String)
					.collect::<Vec<_>>()
					== connection.actor_key,
				StreamError::NotConfigured.build()
			);
			// Validate path canonicalization using the same read boundary.
			let _ = connection.source_request(Method::HEAD, &path)?;
			path
		};
		Ok((handler, path))
	}

	pub async fn stream_status_cbor(&self, source: &[u8]) -> Result<Vec<u8>> {
		let value = self.stream_status(ciborium::from_reader(source)?).await?;
		let mut bytes = Vec::new();
		ciborium::into_writer(&value, &mut bytes)?;
		Ok(bytes)
	}
	pub async fn stream_resume_cbor(&self, source: &[u8]) -> Result<()> {
		self.stream_resume(ciborium::from_reader(source)?).await
	}
	pub async fn stream_status(&self, source: StreamSource) -> Result<Value> {
		let connection = self.stream_connection()?;
		let (handler, path) = self.stream_source(&connection, source)?;
		let checkpoint =
			internal_storage::load_meta_text(self.sql(), &checkpoint_key(&handler, &path)?)
				.await?
				.map(|s| Checkpoint::decode(&s))
				.transpose()?;
		Ok(
			json!({"subscription":handler,"stream":path,"offset":checkpoint.as_ref().map(|c|&c.offset),
            "completed":checkpoint.as_ref().map_or(0, |c|c.completed),
            "deadLetter":checkpoint.as_ref().and_then(|c|c.dead_letter.as_ref()).map(|d|json!({"stream":d.stream,"pending":true,"error":d.delivery_error})),
            "deadLetterStream":connection.dead_letter_path(&handler),
            "failure":checkpoint.as_ref().map_or(Value::Null,|c|c.failure_info(&handler,&path))}),
		)
	}

	pub async fn stream_resume(&self, source: StreamSource) -> Result<()> {
		let awake = self.keep_awake_region();
		let lock = self
			.0
			.stream_lock
			.clone()
			.try_lock_owned()
			.map_err(|_| StreamError::Busy.build())?;
		let connection = self.stream_connection()?;
		let (handler, path) = self.stream_source(&connection, source)?;
		let key = checkpoint_key(&handler, &path)?;
		let mut checkpoint = internal_storage::load_meta_text(self.sql(), &key)
			.await?
			.map(|s| Checkpoint::decode(&s))
			.transpose()?
			.ok_or_else(|| StreamError::NotPaused.build())?;
		ensure!(
			checkpoint.failure.as_ref().is_some_and(|f| f.paused),
			StreamError::NotPaused.build()
		);
		if checkpoint.dead_letter.is_none() {
			checkpoint.failure = None;
		}
		checkpoint.save(self, &key).await?;
		let ctx = self.clone();
		self.register_task(async move {
            let _awake = awake;
            let _lock = lock;
            let cancel = CancellationToken::new();
            let result = async {
                let (headers, _) = checked(connection.source_request(Method::HEAD, &path)?).await?;
                let tail = headers.get("stream-next-offset").context("DS omitted Stream-Next-Offset")?.to_str()?.to_owned();
                ctx.drain_source(&connection, &handler, &WakeStream {path, acked_offset:checkpoint.offset,tail_offset:tail}, &cancel).await
            };
            let shutdown = ctx.shutdown_deadline_token();
            tokio::select! {
                result = result => if let Err(error) = result { tracing::warn!(?error, "Resumed stream awaits redelivery"); },
                _ = shutdown.cancelled() => {},
            }
            cancel.cancel();
        });
		Ok(())
	}

	async fn report_stream_failure(
		&self,
		checkpoint: &mut Checkpoint,
		key: &str,
		handler: &str,
		path: &str,
		cancel: &CancellationToken,
	) -> Result<()> {
		if checkpoint.failure.as_ref().is_some_and(|f| !f.notified) {
			let mut body = Vec::new();
			ciborium::into_writer(&checkpoint.failure_info(handler, path), &mut body)?;
			let (tx, rx) = oneshot::channel();
			self.try_send_actor_event(
				ActorEvent::StreamError {
					subscription: handler.into(),
					body,
					cancel: cancel.clone(),
					reply: tx.into(),
				},
				"stream_error",
			)?;
			if let Err(error) = rx.await.context("Stream error handler stopped")? {
				tracing::warn!(?error, "stream.onError failed; source remains paused");
			}
			checkpoint
				.failure
				.as_mut()
				.expect("failure exists")
				.notified = true;
			checkpoint.save(self, key).await?;
		}
		Ok(())
	}

	async fn drain_source(
		&self,
		connection: &Connection,
		handler: &str,
		stream: &WakeStream,
		cancel: &CancellationToken,
	) -> Result<bool> {
		let key = checkpoint_key(handler, &stream.path)?;
		let mut checkpoint = match internal_storage::load_meta_text(self.sql(), &key).await? {
			Some(raw) => Checkpoint::decode(&raw)?,
			None => Checkpoint {
				offset: stream.acked_offset.clone(),
				completed: 0,
				failure: None,
				dead_letter: None,
			},
		};
		if checkpoint.offset < stream.acked_offset && checkpoint.dead_letter.is_none() {
			checkpoint = Checkpoint {
				offset: stream.acked_offset.clone(),
				completed: 0,
				failure: None,
				dead_letter: None,
			};
		}
		// Complete a persisted handoff before reading the source again.
		if checkpoint.dead_letter.is_some() {
			if !self
				.deliver_dead_letter(connection, &mut checkpoint, &key)
				.await?
			{
				return Ok(false);
			}
			self.finish_stream_record(&mut checkpoint, &key).await?;
		}
		while checkpoint.offset < stream.tail_offset {
			if checkpoint.failure.as_ref().is_some_and(|f| f.paused)
				&& connection.dead_letter_path(handler).is_none()
			{
				self.report_stream_failure(&mut checkpoint, &key, handler, &stream.path, cancel)
					.await?;
				return Ok(false);
			}
			let (headers, body) = checked(
				connection
					.source_request(Method::GET, &stream.path)?
					.query(&[("offset", &checkpoint.offset)]),
			)
			.await?;
			let next = headers
				.get("stream-next-offset")
				.context("DS omitted Stream-Next-Offset")?
				.to_str()?
				.to_owned();
			let messages: Vec<Value> = serde_json::from_slice(&body)?;
			ensure!(
				next > checkpoint.offset && !messages.is_empty(),
				"Stream no longer contains pending input"
			);
			for message in &messages[checkpoint.completed.min(messages.len())..] {
				loop {
					if checkpoint.failure.as_ref().is_some_and(|f| f.paused) {
						self.report_stream_failure(
							&mut checkpoint,
							&key,
							handler,
							&stream.path,
							cancel,
						)
						.await?;
						if let Some(target) = connection.dead_letter_path(handler) {
							self.prepare_dead_letter(
								&mut checkpoint,
								&key,
								handler,
								&stream.path,
								target,
								message,
							)
							.await?;
							if self
								.deliver_dead_letter(connection, &mut checkpoint, &key)
								.await?
							{
								break;
							}
						}
						return Ok(false);
					}
					if let Some(failure) = &checkpoint.failure {
						tokio::time::sleep(Duration::from_millis(
							failure.next_attempt_at.saturating_sub(now_ms()).max(0) as u64,
						))
						.await;
					}
					let mut body = Vec::new();
					ciborium::into_writer(message, &mut body)?;
					let (tx, rx) = oneshot::channel();
					self.try_send_actor_event(
						ActorEvent::StreamMessage {
							subscription: handler.into(),
							body,
							cancel: cancel.clone(),
							reply: tx.into(),
						},
						"stream_message",
					)?;
					match rx.await.context("Stream handler stopped")? {
						Ok(()) => break,
						Err(error) => {
							checkpoint.failed(&error, now_ms());
							checkpoint.save(self, &key).await?;
						}
					}
				}
				self.finish_stream_record(&mut checkpoint, &key).await?;
			}
			checkpoint.offset = next;
			checkpoint.completed = checkpoint.completed.saturating_sub(messages.len());
			checkpoint.save(self, &key).await?;
		}
		Ok(true)
	}
}
