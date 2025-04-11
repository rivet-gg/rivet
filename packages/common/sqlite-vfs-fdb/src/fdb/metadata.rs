use bytes::{BufMut, Bytes, BytesMut};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::utils::FdbVfsError;

/// Metadata associated with a file in the FDB storage
#[derive(Debug, Clone, Copy)]
pub struct FdbFileMetadata {
	/// Unique identifier for this file
	pub file_id: Uuid,
	/// File size in bytes
	pub size: i64,
	/// Time when file was created
	pub created_at: u64,
	/// Time when file was last modified
	pub modified_at: u64,
	/// Page size for this file
	pub page_size: usize,
}

impl FdbFileMetadata {
	pub fn new(page_size: usize) -> Self {
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		Self {
			file_id: Uuid::new_v4(),
			size: 0,
			created_at: now,
			modified_at: now,
			page_size,
		}
	}

	pub fn to_bytes(&self) -> Bytes {
		let mut buf = BytesMut::with_capacity(16 + 8 + 8 + 8 + 4 + 1);

		// Write file_id (16 bytes)
		buf.put_slice(self.file_id.as_bytes());

		// Write size (8 bytes)
		buf.put_i64(self.size);

		// Write created_at (8 bytes)
		buf.put_u64(self.created_at);

		// Write modified_at (8 bytes)
		buf.put_u64(self.modified_at);

		// Write page_size (4 bytes)
		buf.put_u32(self.page_size as u32);

		buf.freeze()
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Self, FdbVfsError> {
		let expected_len = 16 + 8 + 8 + 8 + 4;
		if bytes.len() < expected_len {
			return Err(FdbVfsError::Other(format!(
				"Metadata too short: {} bytes, expected at least {}",
				bytes.len(),
				expected_len
			)));
		}

		let file_id = Uuid::from_slice(&bytes[0..16])
			.map_err(|e| FdbVfsError::Other(format!("Invalid UUID in metadata: {}", e)))?;

		let size = i64::from_be_bytes(bytes[16..24].try_into()
			.map_err(|_| FdbVfsError::Other("Failed to convert size bytes".to_string()))?);

		let created_at = u64::from_be_bytes(bytes[24..32].try_into()
			.map_err(|_| FdbVfsError::Other("Failed to convert created_at bytes".to_string()))?);

		let modified_at = u64::from_be_bytes(bytes[32..40].try_into()
			.map_err(|_| FdbVfsError::Other("Failed to convert modified_at bytes".to_string()))?);

		let page_size = u32::from_be_bytes(bytes[40..44].try_into()
			.map_err(|_| FdbVfsError::Other("Failed to convert page_size bytes".to_string()))?) as usize;

		Ok(Self {
			file_id,
			size,
			created_at,
			modified_at,
			page_size,
		})
	}
}