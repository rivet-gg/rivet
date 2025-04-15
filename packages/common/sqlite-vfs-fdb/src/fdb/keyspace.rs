use bytes::{BufMut, Bytes, BytesMut};
use uuid::Uuid;

/// Key space management for storing file data in FoundationDB
#[derive(Clone)]
pub struct FdbKeySpace {
	// Prefix for all keys in this VFS instance
	prefix: Bytes,
}

impl FdbKeySpace {
	pub fn new(prefix: &[u8]) -> Self {
		tracing::info!("Creating FdbKeySpace with prefix length: {}", prefix.len());
		if prefix.is_empty() {
			tracing::error!("ERROR: Empty prefix provided to FdbKeySpace::new!");
		} else {
			tracing::info!("Prefix data: {:?}", prefix);
		}
		
		Self {
			prefix: Bytes::copy_from_slice(prefix),
		}
	}
	
	// Method for getting prefix length
	pub fn prefix_len(&self) -> usize {
		self.prefix.len()
	}
	
	// Method to check if prefix is empty
	pub fn prefix_is_empty(&self) -> bool {
		self.prefix.is_empty()
	}

	/// Key for file metadata
	pub fn metadata_key(&self, path: &str) -> Bytes {
		// Validate input path - truncate if too long to prevent overflow
		// SQLite paths typically shouldn't be extremely long anyway
		const MAX_PATH_LEN: usize = 4096; // Reasonable maximum path length
		let safe_path = if path.len() > MAX_PATH_LEN {
			&path[0..MAX_PATH_LEN]
		} else {
			path
		};
		
		tracing::info!("Creating metadata key for path: {}", path);
		let prefix_len = self.prefix.len();
		let prefix_empty = self.prefix.is_empty();
		tracing::info!("Prefix check - length: {}, is_empty: {}", prefix_len, prefix_empty);
		
		if !prefix_empty && prefix_len > 0 {
			tracing::info!("Prefix data: {:?}", &self.prefix[..]);
		} else {
			tracing::error!("Warning: Empty or invalid prefix detected!");
		}
		
		let mut buf = BytesMut::with_capacity(self.prefix.len() + 1 + safe_path.len());
		buf.put_slice(&self.prefix);
		buf.put_u8(0); // Metadata key type
		buf.put_slice(safe_path.as_bytes());
		buf.freeze()
	}

	/// Key for file page data
	pub fn page_key(&self, file_id: &Uuid, page_num: u32) -> Bytes {
		let mut buf = BytesMut::with_capacity(self.prefix.len() + 1 + 16 + 4);
		buf.put_slice(&self.prefix);
		buf.put_u8(1); // Page key type
		buf.put_slice(file_id.as_bytes());
		buf.put_u32(page_num);
		buf.freeze()
	}

	/// Key for lock information
	pub fn lock_key(&self, path: &str) -> Bytes {
		// Validate input path - truncate if too long to prevent overflow
		// SQLite paths typically shouldn't be extremely long anyway
		const MAX_PATH_LEN: usize = 4096; // Reasonable maximum path length
		let safe_path = if path.len() > MAX_PATH_LEN {
			&path[0..MAX_PATH_LEN]
		} else {
			path
		};
		
		let mut buf = BytesMut::with_capacity(self.prefix.len() + 1 + safe_path.len());
		buf.put_slice(&self.prefix);
		buf.put_u8(2); // Lock key type
		buf.put_slice(safe_path.as_bytes());
		buf.freeze()
	}
}
