//! Durable handoff and explicit, idempotent primary-input redrive.
use super::*;
use rivetkit_actor_persist::{
	generated::stream_redrive_v1::Receipt,
	stream_redrive::{StreamRedrive, VERSION},
};

impl Connection {
	pub(super) fn dead_letter_path(&self, handler: &str) -> Option<String> {
		self.dead_letters.get(handler).map(|config| {
			config
				.stream
				.clone()
				.unwrap_or_else(|| format!("{}/dead-letters/{handler}", self.path))
		})
	}
	async fn append_once(
		&self,
		path: &str,
		producer: &str,
		body: &Value,
		token: Option<&str>,
	) -> Result<String> {
		checked(
			self.source_request_with_token(Method::PUT, path, token)?
				.header("content-type", "application/json"),
		)
		.await?;
		let (headers, _) = checked(
			self.source_request_with_token(Method::POST, path, token)?
				.header("Producer-Id", producer)
				.header("Producer-Epoch", "0")
				.header("Producer-Seq", "0")
				.json(&json!([body])),
		)
		.await?;
		Ok(headers
			.get("stream-next-offset")
			.context("DS omitted Stream-Next-Offset")?
			.to_str()?
			.to_owned())
	}
}

impl ActorContext {
	pub(super) async fn prepare_dead_letter(
		&self,
		checkpoint: &mut Checkpoint,
		key: &str,
		handler: &str,
		path: &str,
		target: String,
		message: &Value,
	) -> Result<()> {
		ensure!(
			target != path,
			"Dead-letter destination cannot be its source"
		);
		let id = format!(
			"{:x}",
			Sha256::digest(serde_json::to_vec(&(
				self.actor_id(),
				handler,
				path,
				&checkpoint.offset,
				checkpoint.completed
			))?)
		);
		let envelope = json!({"id":id,"actor":{"id":self.actor_id(),"name":self.name(),"key":self.key()},
            "subscription":handler,"source":{"stream":path,"offset":checkpoint.offset,"messageIndex":checkpoint.completed},
            "body":message,"failure":checkpoint.failure_info(handler,path)});
		checkpoint.dead_letter = Some(wire::DeadLetter {
			stream: target,
			producer_id: format!("rivet-dlq-{id}"),
			envelope: serde_json::to_vec(&envelope)?,
			delivery_error: None,
		});
		checkpoint.save(self, key).await
	}
	pub(super) async fn deliver_dead_letter(
		&self,
		connection: &Connection,
		checkpoint: &mut Checkpoint,
		key: &str,
	) -> Result<bool> {
		let pending = checkpoint
			.dead_letter
			.as_mut()
			.context("Missing pending handoff")?;
		let result = connection
			.append_once(
				&pending.stream,
				&pending.producer_id,
				&serde_json::from_slice(&pending.envelope)?,
				connection.token.as_deref(),
			)
			.await;
		if let Err(error) = result {
			pending.delivery_error = Some(format!("{error:#}").chars().take(4096).collect());
			checkpoint.save(self, key).await?;
			return Ok(false);
		}
		Ok(true)
	}
	pub(super) async fn finish_stream_record(
		&self,
		checkpoint: &mut Checkpoint,
		key: &str,
	) -> Result<()> {
		self.request_save_and_wait(RequestSaveOpts {
			immediate: true,
			..Default::default()
		})
		.await?;
		checkpoint.failure = None;
		checkpoint.dead_letter = None;
		checkpoint.completed += 1;
		checkpoint.save(self, key).await
	}
	pub async fn stream_redrive_cbor(&self, request: &[u8]) -> Result<Vec<u8>> {
		let request: Value = ciborium::from_reader(request)?;
		let token = self.stream_connection()?.token;
		let value = self
			.stream_redrive_authenticated(request, token.as_deref())
			.await?;
		let mut bytes = Vec::new();
		ciborium::into_writer(&value, &mut bytes)?;
		Ok(bytes)
	}
	/// Appends original input at the primary stream tail. Requires an explicit retry identity.
	pub async fn stream_redrive_authenticated(
		&self,
		request: Value,
		token: Option<&str>,
	) -> Result<Value> {
		let token = token
			.filter(|t| !t.is_empty())
			.context("Redrive requires caller credentials")?;
		let _awake = self.keep_awake_region();
		// Serialize receipt checks with other redrives and stream processing.
		let _lock = self
			.0
			.stream_lock
			.clone()
			.try_lock_owned()
			.map_err(|_| StreamError::Busy.build())?;
		let connection = self.stream_connection()?;
		ensure!(connection.primary, StreamError::NotConfigured.build());
		let letter = request
			.get("message")
			.context("Missing dead-letter message")?;
		ensure!(
			letter["actor"]["id"].as_str() == Some(self.actor_id())
				&& letter["actor"]["name"].as_str() == Some(self.name()),
			"Dead letter belongs to another actor"
		);
		ensure!(
			letter["subscription"] == "primary" && letter["source"]["stream"] == connection.path,
			"Only primary-input redrive is supported; shared sources require explicit republishing"
		);
		let failure_id = letter["id"]
			.as_str()
			.filter(|id| id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit()))
			.context("Invalid failure ID")?;
		let identity = request["idempotencyKey"]
			.as_str()
			.filter(|id| !id.is_empty() && id.len() <= 256)
			.context("An idempotencyKey of 1–256 bytes is required")?;
		let body = letter.get("body").context("Missing original body")?;
		let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(body)?));
		let producer = format!(
			"rivet-redrive-{:x}",
			Sha256::digest(serde_json::to_vec(&(self.actor_id(), identity))?)
		);
		let key = format!("stream_receipt_{producer}");
		if let Some(raw) = internal_storage::load_meta_text(self.sql(), &key).await? {
			let receipt =
				StreamRedrive::deserialize_with_embedded_version(&URL_SAFE_NO_PAD.decode(raw)?)?;
			ensure!(
				receipt.failure_id == failure_id && receipt.body_digest == digest,
				"Idempotency key was already used for different input"
			);
			if !receipt.next_offset.is_empty() {
				return Ok(receipt_value(&receipt));
			}
		}
		// Persist the identity before append so a crash cannot permit different input under the same key.
		let pending = Receipt {
			failure_id: failure_id.into(),
			idempotency_key: identity.into(),
			stream: connection.path.clone(),
			next_offset: String::new(),
			body_digest: digest.clone(),
			created_at: now_ms(),
		};
		let bytes = StreamRedrive::wrap_latest(pending).serialize_with_embedded_version(VERSION)?;
		internal_storage::persist_meta_text(self.sql(), &key, &URL_SAFE_NO_PAD.encode(bytes))
			.await?;
		let next_offset = connection
			.append_once(&connection.path, &producer, body, Some(token))
			.await?;
		let receipt = Receipt {
			failure_id: failure_id.into(),
			idempotency_key: identity.into(),
			stream: connection.path.clone(),
			next_offset,
			body_digest: digest,
			created_at: now_ms(),
		};
		let value = receipt_value(&receipt);
		let bytes = StreamRedrive::wrap_latest(receipt).serialize_with_embedded_version(VERSION)?;
		internal_storage::persist_meta_text(self.sql(), &key, &URL_SAFE_NO_PAD.encode(bytes))
			.await?;
		Ok(value)
	}
}
fn receipt_value(r: &Receipt) -> Value {
	json!({"failureId":r.failure_id,"idempotencyKey":r.idempotency_key,"stream":r.stream,"nextOffset":r.next_offset,"createdAt":r.created_at})
}

#[cfg(test)]
#[path = "../../../tests/unit/stream/dead_letter.rs"]
mod tests;
