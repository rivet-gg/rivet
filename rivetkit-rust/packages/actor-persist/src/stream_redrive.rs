use crate::generated::stream_redrive_v1;
use anyhow::{Result, bail};
use vbare::OwnedVersionedData;
pub const VERSION: u16 = 1;
pub struct StreamRedrive(pub stream_redrive_v1::Receipt);
impl OwnedVersionedData for StreamRedrive {
	type Latest = stream_redrive_v1::Receipt;
	fn wrap_latest(data: Self::Latest) -> Self {
		Self(data)
	}
	fn unwrap_latest(self) -> Result<Self::Latest> {
		Ok(self.0)
	}
	fn deserialize_version(payload: &[u8], version: u16) -> Result<Self> {
		match version {
			1 => Ok(Self(serde_bare::from_slice(payload)?)),
			_ => bail!("Unknown redrive receipt version"),
		}
	}
	fn serialize_version(self, version: u16) -> Result<Vec<u8>> {
		match version {
			1 => Ok(serde_bare::to_vec(&self.0)?),
			_ => bail!("Unknown redrive receipt version"),
		}
	}
	fn deserialize_converters() -> Vec<impl Fn(Self) -> Result<Self>> {
		Vec::<fn(Self) -> Result<Self>>::new()
	}
	fn serialize_converters() -> Vec<impl Fn(Self) -> Result<Self>> {
		Vec::<fn(Self) -> Result<Self>>::new()
	}
}
