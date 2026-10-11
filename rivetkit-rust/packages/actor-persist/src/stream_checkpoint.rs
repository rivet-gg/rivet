use crate::generated::{stream_checkpoint_v1 as v1, stream_checkpoint_v2 as v2};
use anyhow::{Result, bail};
use vbare::OwnedVersionedData;

pub enum StreamCheckpoint {
	V1(v1::Data),
	V2(v2::Data),
}
impl OwnedVersionedData for StreamCheckpoint {
	type Latest = v2::Data;
	fn wrap_latest(data: Self::Latest) -> Self {
		Self::V2(data)
	}
	fn unwrap_latest(self) -> Result<Self::Latest> {
		match self {
			Self::V2(data) => Ok(data),
			_ => bail!("Version not latest"),
		}
	}
	fn deserialize_version(payload: &[u8], version: u16) -> Result<Self> {
		match version {
			1 => Ok(Self::V1(serde_bare::from_slice(payload)?)),
			2 => Ok(Self::V2(serde_bare::from_slice(payload)?)),
			_ => bail!("Unknown stream checkpoint version"),
		}
	}
	fn serialize_version(self, version: u16) -> Result<Vec<u8>> {
		match (self, version) {
			(Self::V1(data), 1) => Ok(serde_bare::to_vec(&data)?),
			(Self::V2(data), 2) => Ok(serde_bare::to_vec(&data)?),
			_ => bail!("Unknown stream checkpoint version"),
		}
	}
	fn deserialize_converters() -> Vec<impl Fn(Self) -> Result<Self>> {
		vec![Self::v1_to_v2]
	}
	fn serialize_converters() -> Vec<impl Fn(Self) -> Result<Self>> {
		vec![Self::v2_to_v1]
	}
}

impl StreamCheckpoint {
	fn v1_to_v2(self) -> Result<Self> {
		let Self::V1(data) = self else {
			bail!("Unexpected version")
		};
		Ok(Self::V2(v2::Data {
			offset: data.offset,
			completed: data.completed,
			dead_letter: None,
			failure: data.failure.map(|f| v2::Failure {
				attempts: f.attempts,
				first_failed_at: f.first_failed_at,
				last_failed_at: f.last_failed_at,
				next_attempt_at: f.next_attempt_at,
				paused: f.paused,
				notified: f.notified,
				error: f.error,
			}),
		}))
	}
	fn v2_to_v1(self) -> Result<Self> {
		let Self::V2(data) = self else {
			bail!("Unexpected version")
		};
		anyhow::ensure!(
			data.dead_letter.is_none(),
			"Cannot downgrade a pending dead-letter handoff"
		);
		Ok(Self::V1(v1::Data {
			offset: data.offset,
			completed: data.completed,
			failure: data.failure.map(|f| v1::Failure {
				attempts: f.attempts,
				first_failed_at: f.first_failed_at,
				last_failed_at: f.last_failed_at,
				next_attempt_at: f.next_attempt_at,
				paused: f.paused,
				notified: f.notified,
				error: f.error,
			}),
		}))
	}
}
