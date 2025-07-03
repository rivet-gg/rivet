use anyhow::*;
use prost::Message;
use tokio_util::codec::LengthDelimitedCodec;

// Include generated protobuf code
pub mod proto {
	pub mod kv {
		include!(concat!(env!("OUT_DIR"), "/rivet.pegboard.kv.rs"));
	}

	pub mod runner_protocol {
		include!(concat!(
			env!("OUT_DIR"),
			"/rivet.pegboard.runner_protocol.rs"
		));
	}
	pub use runner_protocol::*;
}

// Small subset of the ToRunner enum that gets proxied to the actor
#[derive(Debug, Clone)]
pub enum ToActor {
	StateUpdate {
		state: proto::ActorState,
	},
	Kv(proto::kv::Request),
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

pub fn encode_frame<T: Message>(payload: &T) -> Result<Vec<u8>> {
	let mut buf = Vec::with_capacity(4 + payload.encoded_len());

	buf.extend_from_slice(&[0u8; 4]); // header (currently unused)

	payload.encode(&mut buf)?;

	Ok(buf)
}

pub fn decode_frame<T: Message + Default>(frame: &[u8]) -> Result<([u8; 4], T)> {
	ensure!(frame.len() >= 4, "Frame too short");

	// Extract the header (first 4 bytes)
	let header = [frame[0], frame[1], frame[2], frame[3]];

	// Decode the rest of the frame (payload after the header)
	let payload = T::decode(&frame[4..])?;

	Ok((header, payload))
}
