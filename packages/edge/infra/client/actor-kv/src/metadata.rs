use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, ::prost::Message, Deserialize, Serialize)]
pub struct Metadata {
	#[prost(bytes = "vec", tag = "1")]
	pub kv_version: Vec<u8>,
	#[prost(int64, tag = "2")]
	pub create_ts: i64,
}
