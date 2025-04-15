use bytes::{BufMut, Bytes, BytesMut};
use uuid::Uuid;

/// Maximum path length for SQLite paths 
const MAX_PATH_LEN: usize = 4096;

/// Key space management for storing file data in FoundationDB
#[derive(Clone)]
pub struct FdbKeySpace {
	// Prefix for all keys in this VFS instance
	prefix: Bytes,
}

impl FdbKeySpace {
	pub fn new(prefix: &[u8]) -> Self {
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
		let safe_path = if path.len() > MAX_PATH_LEN {
			&path[0..MAX_PATH_LEN]
		} else {
			path
		};
		
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
	
	/// Key for WAL frame data
	/// Using salt values in the key ensures frames are stored in the same order
	/// as they appear in the WAL file, which is important for recovery
	pub fn wal_frame_key(&self, file_id: &Uuid, salt1: u32, salt2: u32, frame_idx: u32) -> Bytes {
		let mut buf = BytesMut::with_capacity(self.prefix.len() + 1 + 16 + 4 + 4 + 4);
		buf.put_slice(&self.prefix);
		buf.put_u8(3); // WAL frame key type
		buf.put_slice(file_id.as_bytes());
		buf.put_u32(salt1);
		buf.put_u32(salt2);
		buf.put_u32(frame_idx); // Frame index to keep order within same salt values
		buf.freeze()
	}

	/// Key for WAL header data
	pub fn wal_header_key(&self, file_id: &Uuid) -> Bytes {
		let mut buf = BytesMut::with_capacity(self.prefix.len() + 1 + 16);
		buf.put_slice(&self.prefix);
		buf.put_u8(4); // WAL header key type
		buf.put_slice(file_id.as_bytes());
		buf.freeze()
	}
	
	/// Index for WAL frames to allow easy lookup by frame number
	/// This is needed to find frames without knowing their salt values ahead of time
	pub fn wal_frame_index_key(&self, file_id: &Uuid, frame_idx: u32) -> Bytes {
		let mut buf = BytesMut::with_capacity(self.prefix.len() + 1 + 16 + 4);
		buf.put_slice(&self.prefix);
		buf.put_u8(5); // WAL frame index key type
		buf.put_slice(file_id.as_bytes());
		buf.put_u32(frame_idx);
		buf.freeze()
	}
}
