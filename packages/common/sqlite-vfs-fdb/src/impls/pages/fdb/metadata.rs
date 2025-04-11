use bytes::{BufMut, Bytes, BytesMut};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::impls::pages::utils::{CompressionType, FdbVfsError};

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
	/// Compression type used for this file
	pub compression_type: CompressionType,
}

impl FdbFileMetadata {
	pub fn new(page_size: usize) -> Self {
		Self::with_compression(page_size, CompressionType::None)
	}
	
	pub fn with_compression(page_size: usize, compression_type: CompressionType) -> Self {
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
			compression_type,
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
		
		// Write compression_type (1 byte)
		buf.put_u8(self.compression_type as u8);

		buf.freeze()
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Self, FdbVfsError> {
		let expected_len = 16 + 8 + 8 + 8 + 4 + 1;
		if bytes.len() < expected_len {
			return Err(FdbVfsError::Other(format!(
				"Metadata too short: {} bytes, expected at least {}",
				bytes.len(),
				expected_len
			)));
		}

		let file_id = Uuid::from_slice(&bytes[0..16])
			.map_err(|e| FdbVfsError::Other(format!("Invalid UUID in metadata: {}", e)))?;

		let size = i64::from_be_bytes([
			bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
		]);

		let created_at = u64::from_be_bytes([
			bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
		]);

		let modified_at = u64::from_be_bytes([
			bytes[32], bytes[33], bytes[34], bytes[35], bytes[36], bytes[37], bytes[38], bytes[39],
		]);

		let page_size = u32::from_be_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as usize;
		
		// Read compression type - handle older metadata format
		let compression_type = if bytes.len() > 44 {
			CompressionType::from(bytes[44])
		} else {
			CompressionType::None
		};

		Ok(Self {
			file_id,
			size,
			created_at,
			modified_at,
			page_size,
			compression_type,
		})
	}
}
