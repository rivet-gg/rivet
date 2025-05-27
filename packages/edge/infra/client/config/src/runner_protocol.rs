use std::io::{Write, Cursor};

use anyhow::*;
use pegboard::protocol;
use tokio_util::codec::LengthDelimitedCodec;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ToManager {
	ActorStateUpdate {
		actor_id: rivet_util::Id,
		generation: u32,
		state: ActorState,
	},
	Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ToRunner {
	Pong,
	Close {
		reason: Option<String>,
	},

	StartActor {
		actor_id: rivet_util::Id,
		generation: u32,
		env: protocol::HashableMap<String, String>,
		metadata: protocol::Raw<protocol::ActorMetadata>,
	},
	SignalActor {
		actor_id: rivet_util::Id,
		generation: u32,
		signal: i32,
		persist_storage: bool,
	},
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ActorState {
	Running,
	Exited { exit_code: Option<i32> },
}

pub fn codec() -> LengthDelimitedCodec {
	LengthDelimitedCodec::builder()
		.length_field_type::<u32>()
		.length_field_length(4)
		// No offset
		.length_field_offset(0)
		// Header length is not included in the length calculation
		.length_adjustment(4)
		// Skip length, but header is included in the returned bytes
		.num_skip(4)
		.new_codec()
}

pub fn encode_frame<T: Serialize>(payload: &T) -> Result<Vec<u8>> {
	let mut buf = Vec::with_capacity(4);
	let mut cursor = Cursor::new(&mut buf);

	cursor.write(&[0u8; 4])?; // header (currently unused)

	serde_json::to_writer(&mut cursor, payload)?;

	cursor.flush()?;

	Ok(buf)
}

pub fn decode_frame<T: DeserializeOwned>(frame: &[u8]) -> Result<([u8; 4], T)> {
	ensure!(frame.len() >= 4, "Frame too short");

	// Extract the header (first 4 bytes)
	let header = [frame[0], frame[1], frame[2], frame[3]];

	// Deserialize the rest of the frame (payload after the header)
	let payload = serde_json::from_slice(&frame[4..])?;

	Ok((header, payload))
}
