use bytes::BytesMut;
use foundationdb::{Database, FdbBindingError, FdbError};
use libsqlite3_sys::*;
use log::{debug, error};
use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_void};
use std::ptr;
use std::slice;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::metadata::FdbFileMetadata;
use crate::utils::{run_fdb_tx, LockState, SQLITE_IOERR, SQLITE_OK};
use crate::vfs::FdbVfs;

/// FdbFile is the implementation of a SQLite file in FoundationDB
#[repr(C)]
pub struct FdbFile {
	/// Base SQLite file structure that must be first
	pub base: sqlite3_file,
	/// Extension with all Rust-specific data, properly initialized
	pub ext: MaybeUninit<FdbFileExt>,
}

/// Extension part of FdbFile that contains all non-C data
pub struct FdbFileExt {
	/// VFS instance that created this file
	pub vfs: *const FdbVfs,
	/// Path to this file
	pub path: String,
	/// File flags (read/write/create)
	pub flags: c_int,
	/// Current lock state
	pub lock_state: LockState,
	/// File metadata
	pub metadata: FdbFileMetadata,
	/// Methods for this file
	pub methods: *const sqlite3_io_methods,
	/// Reference to the FDB database for operations
	pub db: Arc<Database>,
	/// Whether file is open
	pub is_open: bool,
}

/// The I/O methods for FdbFile
pub static FDB_IO_METHODS: sqlite3_io_methods = sqlite3_io_methods {
	iVersion: 1,
	xClose: Some(fdb_file_close),
	xRead: Some(fdb_file_read),
	xWrite: Some(fdb_file_write),
	xTruncate: Some(fdb_file_truncate),
	xSync: Some(fdb_file_sync),
	xFileSize: Some(fdb_file_size),
	xLock: Some(fdb_file_lock),
	xUnlock: Some(fdb_file_unlock),
	xCheckReservedLock: Some(fdb_file_check_reserved_lock),
	xFileControl: Some(fdb_file_control),
	xSectorSize: Some(fdb_file_sector_size),
	xDeviceCharacteristics: Some(fdb_file_device_characteristics),
	xShmMap: None,
	xShmLock: None,
	xShmBarrier: None,
	xShmUnmap: None,
	xFetch: None,
	xUnfetch: None,
};

// Implementation of SQLite I/O Methods for FdbFile
pub unsafe extern "C" fn fdb_file_close(file: *mut sqlite3_file) -> c_int {
	debug!("fdb_file_close called");

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			error!("Null file pointer in fdb_file_close");
			return SQLITE_IOERR;
		}
	};

	// Extract ext to drop it
	let ext = std::mem::replace(&mut fdb_file.ext, MaybeUninit::uninit());
	let _ext = ext.assume_init();

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_read(
	file: *mut sqlite3_file,
	buf: *mut c_void,
	count: c_int,
	offset: i64,
) -> c_int {
	debug!("fdb_file_read called: count={}, offset={}", count, offset);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			error!("Null file pointer in fdb_file_read");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		return SQLITE_IOERR;
	}

	let vfs = match ext.vfs.as_ref() {
		Some(v) => v,
		None => {
			error!("Null VFS pointer in fdb_file_read");
			return SQLITE_IOERR;
		}
	};

	// Extract all data we need before entering the closure
	let file_id = ext.metadata.file_id;
	let page_size = ext.metadata.page_size as i64;
	let file_size = ext.metadata.size;

	// Zero the buffer if reading beyond EOF
	if offset >= file_size {
		ptr::write_bytes(buf, 0, count as usize);
		return SQLITE_IOERR_SHORT_READ;
	}

	// Calculate the range of pages to read
	let start_page = (offset / page_size) as u32;
	let end_offset = std::cmp::min(offset + (count as i64), file_size);
	let end_page = ((end_offset - 1) / page_size) as u32;

	// Create a buffer to store the data we read
	// We'll copy this to the output buffer afterwards to avoid
	// sending raw pointers across threads
	let data_buffer = vec![0u8; count as usize];

	let keyspace = vfs.keyspace.clone();
	let db = vfs.db.clone();

	// Read data from FoundationDB
	let read_result = run_fdb_tx(&db, move |tx| {
		// Clone everything that will be moved into the async block
		let file_id = file_id; // This is a UUID which is Copy
		let page_size = page_size; // This is a primitive which is Copy
		let offset = offset; // This is a primitive which is Copy
		let end_page = end_page; // This is a primitive which is Copy
		let start_page = start_page; // This is a primitive which is Copy
		let keyspace_clone = keyspace.clone(); // Clone the keyspace for ownership
		let mut data_buffer = data_buffer.clone(); // Clone instead of moving, make it mutable

		async move {
			let mut bytes_read = 0;

			// Read each page that contains data we need
			for page_num in start_page..=end_page {
				let page_key = keyspace_clone.page_key(&file_id, page_num);

				// Read page data from FoundationDB
				let page_data_result = tx.get(&page_key, false).await?;

				if let Some(page_data) = page_data_result {
					// Calculate the part of this page that we need to copy
					let page_offset = page_num as i64 * page_size;
					let relative_start = if offset > page_offset {
						(offset - page_offset) as usize
					} else {
						0
					};

					let page_bytes_available = if relative_start < page_data.len() {
						page_data.len() - relative_start
					} else {
						0
					};

					let bytes_to_copy =
						std::cmp::min(page_bytes_available, count as usize - bytes_read);

					if bytes_to_copy > 0 {
						// Copy data to our temporary buffer
						let dest_start = if page_offset < offset {
							0
						} else {
							(page_offset - offset) as usize
						};

						if dest_start + bytes_to_copy <= data_buffer.len() {
							data_buffer[dest_start..dest_start + bytes_to_copy].copy_from_slice(
								&page_data[relative_start..relative_start + bytes_to_copy],
							);

							bytes_read += bytes_to_copy;
						}
					}
				}
			}

			// Return both the number of bytes we read and the buffer
			Ok((bytes_read, data_buffer))
		}
	});

	match read_result {
		Ok((bytes_read, data_buffer)) => {
			// Now copy the data from our buffer to the output buffer
			let buf_slice = slice::from_raw_parts_mut(buf as *mut u8, count as usize);
			buf_slice[..bytes_read].copy_from_slice(&data_buffer[..bytes_read]);

			if bytes_read < count as usize {
				// Short read, not an error but not a full read
				SQLITE_IOERR_SHORT_READ
			} else {
				// Full read
				SQLITE_OK
			}
		}
		Err(e) => {
			error!("Error reading file: {}", e);
			SQLITE_IOERR
		}
	}
}

pub unsafe extern "C" fn fdb_file_write(
	file: *mut sqlite3_file,
	buf: *const c_void,
	count: c_int,
	offset: i64,
) -> c_int {
	debug!("fdb_file_write called: count={}, offset={}", count, offset);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			error!("Null file pointer in fdb_file_write");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		return SQLITE_IOERR;
	}

	let vfs = match ext.vfs.as_ref() {
		Some(v) => v,
		None => {
			error!("Null VFS pointer in fdb_file_write");
			return SQLITE_IOERR;
		}
	};

	// Copy data from raw pointer to a safe buffer
	let buf_data = unsafe { slice::from_raw_parts(buf as *const u8, count as usize).to_vec() };

	// Extract all needed data before entering the closure
	let file_id = ext.metadata.file_id;
	let page_size = ext.metadata.page_size as i64;
	let current_size = ext.metadata.size;
	let file_path = ext.path.clone();

	// Calculate the range of pages to write
	let start_page = (offset / page_size) as u32;
	let end_offset = offset + (count as i64);
	let end_page = ((end_offset - 1) / page_size) as u32;

	// Get references to VFS data
	let keyspace = vfs.keyspace.clone();
	let db = vfs.db.clone();

	// Write data to FoundationDB
	let write_result = run_fdb_tx(&db, move |tx| {
		// Clone everything that will be moved into the async block
		let file_id = file_id; // This is a UUID which is Copy
		let page_size = page_size; // This is a primitive which is Copy
		let current_size = current_size; // This is a primitive which is Copy
		let file_path_clone = file_path.clone(); // Clone for ownership
		let keyspace_clone = keyspace.clone(); // Clone for ownership
		let buf_data_clone = buf_data.clone(); // Clone for ownership
		let offset = offset; // This is a primitive which is Copy
		let end_offset = end_offset; // This is a primitive which is Copy
		let start_page = start_page; // This is a primitive which is Copy
		let end_page = end_page; // This is a primitive which is Copy

		async move {
			// Write each page
			for page_num in start_page..=end_page {
				let page_key = keyspace_clone.page_key(&file_id, page_num);

				// Calculate the portion of the buffer that belongs to this page
				let page_offset = page_num as i64 * page_size;
				let page_end = page_offset + page_size;

				// Calculate the overlap between the buffer and this page
				let overlap_start = std::cmp::max(offset, page_offset);
				let overlap_end = std::cmp::min(end_offset, page_end);

				if overlap_start < overlap_end {
					// Calculate buffer indices
					let buf_start = (overlap_start - offset) as usize;
					let buf_end = (overlap_end - offset) as usize;

					// If we're doing a partial page write, we need to read the existing page first
					// and merge our changes with the existing data
					if overlap_start > page_offset || overlap_end < page_end {
						let existing_page_result = tx.get(&page_key, false).await?;

						let mut page_data = match existing_page_result {
							Some(data) => {
								// Use our safe helper to create a properly sized BytesMut
								let mut safe_bytes = crate::utils::create_safe_bytes(page_size as usize);
								// Copy existing data if any
								if !data.is_empty() {
									let copy_len = std::cmp::min(data.len(), safe_bytes.len());
									safe_bytes[..copy_len].copy_from_slice(&data[..copy_len]);
								}
								safe_bytes
							}
							None => {
								// New page, initialize with zeros using our safe helper
								crate::utils::create_safe_bytes(page_size as usize)
							}
						};

						// Calculate the offset within the page
						let page_buf_start = (overlap_start - page_offset) as usize;
						let page_buf_end = (overlap_end - page_offset) as usize;

						// Copy the buffer portion to the page data
						page_data[page_buf_start..page_buf_end]
							.copy_from_slice(&buf_data_clone[buf_start..buf_end]);

						// Write the page back to FoundationDB
						tx.set(&page_key, &page_data);
					} else {
						// Full page write, just use the buffer data directly
						tx.set(&page_key, &buf_data_clone[buf_start..buf_end]);
					}
				}
			}

			// Update the file size in metadata if needed
			let mut new_size = current_size;
			let mut updated_modified_at = None;

			if end_offset > current_size {
				new_size = end_offset;
				updated_modified_at = Some(
					SystemTime::now()
						.duration_since(UNIX_EPOCH)
						.unwrap_or_default()
						.as_secs(),
				);
			}

			// If the size changed, update metadata
			if new_size != current_size {
				// Create new metadata
				let mut updated_metadata = FdbFileMetadata {
					file_id,
					size: new_size,
					page_size: page_size as usize,
					// Placeholder values, will be updated from closure
					created_at: 0,
					modified_at: 0,
				};

				// Get existing metadata to preserve creation time and update modification time
				let metadata_key = keyspace_clone.metadata_key(&file_path_clone);
				let metadata_result = tx.get(&metadata_key, false).await?;

				if let Some(metadata_bytes) = metadata_result {
					if let Ok(old_metadata) = FdbFileMetadata::from_bytes(&metadata_bytes) {
						updated_metadata.created_at = old_metadata.created_at;
						updated_metadata.modified_at = if let Some(new_time) = updated_modified_at {
							new_time
						} else {
							old_metadata.modified_at
						};
					}
				}

				// Write updated metadata to FoundationDB
				let metadata_bytes = updated_metadata.to_bytes();
				tx.set(&metadata_key, &metadata_bytes);
			}

			Ok(new_size)
		}
	});

	match write_result {
		Ok(new_size) => {
			// Update the in-memory file size
			(*fdb_file).ext.assume_init_mut().metadata.size = new_size;
			SQLITE_OK
		}
		Err(e) => {
			error!("Error writing file: {}", e);
			SQLITE_IOERR
		}
	}
}

pub unsafe extern "C" fn fdb_file_truncate(file: *mut sqlite3_file, size: i64) -> c_int {
	debug!("fdb_file_truncate called: size={}", size);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			error!("Null file pointer in fdb_file_truncate");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		return SQLITE_IOERR;
	}

	let vfs = match ext.vfs.as_ref() {
		Some(v) => v,
		None => {
			error!("Null VFS pointer in fdb_file_truncate");
			return SQLITE_IOERR;
		}
	};

	// If size is larger than current size, do nothing (SQLite handles extension separately)
	let current_size = ext.metadata.size;
	if size >= current_size {
		return SQLITE_OK;
	}

	// Extract all needed data before entering the closure
	let file_id = ext.metadata.file_id;
	let page_size = ext.metadata.page_size as i64;
	let file_path = ext.path.clone();

	// Calculate which pages to keep and which to delete
	let pages_to_keep = ((size + page_size - 1) / page_size) as u32;
	let pages_to_delete = ((current_size + page_size - 1) / page_size) as u32 - pages_to_keep;

	// Get references to VFS data
	let keyspace = vfs.keyspace.clone();
	let db = vfs.db.clone();

	// Truncate the file in FoundationDB
	let truncate_result = run_fdb_tx(&db, move |tx| {
		// Clone everything that will be moved into the async block
		let file_id = file_id; // This is a UUID which is Copy
		let file_path_clone = file_path.clone(); // Clone for ownership
		let keyspace_clone = keyspace.clone(); // Clone for ownership
		let pages_to_keep = pages_to_keep; // This is a primitive which is Copy
		let pages_to_delete = pages_to_delete; // This is a primitive which is Copy
		let size = size; // This is a primitive which is Copy

		async move {
			// Get existing metadata
			let metadata_key = keyspace_clone.metadata_key(&file_path_clone);
			let metadata_result = tx.get(&metadata_key, false).await?;

			// Create updated metadata to save
			let updated_metadata = if let Some(metadata_bytes) = metadata_result {
				match FdbFileMetadata::from_bytes(&metadata_bytes) {
					Ok(mut metadata) => {
						metadata.size = size;
						metadata.modified_at = SystemTime::now()
							.duration_since(UNIX_EPOCH)
							.unwrap_or_default()
							.as_secs();
						metadata
					}
					Err(_) => {
						// This shouldn't happen but handle it gracefully
						return Err(FdbBindingError::NonRetryableFdbError(FdbError::from_code(
							1,
						)));
					}
				}
			} else {
				// This shouldn't happen but handle it gracefully
				return Err(FdbBindingError::NonRetryableFdbError(FdbError::from_code(
					1,
				)));
			};

			// Save updated metadata
			let metadata_bytes = updated_metadata.to_bytes();
			tx.set(&metadata_key, &metadata_bytes);

			// Delete unnecessary pages
			for page_num in pages_to_keep..pages_to_keep + pages_to_delete {
				let page_key = keyspace_clone.page_key(&file_id, page_num);
				tx.clear(&page_key);
			}

			Ok(updated_metadata)
		}
	});

	match truncate_result {
		Ok(updated_metadata) => {
			// Update the in-memory metadata
			(*fdb_file).ext.assume_init_mut().metadata = updated_metadata;
			SQLITE_OK
		}
		Err(e) => {
			error!("Error truncating file: {}", e);
			SQLITE_IOERR
		}
	}
}

pub unsafe extern "C" fn fdb_file_sync(_file: *mut sqlite3_file, _flags: c_int) -> c_int {
	// FoundationDB is always synced, so we don't need to do anything
	debug!("fdb_file_sync called");
	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_size(file: *mut sqlite3_file, size_out: *mut i64) -> c_int {
	debug!("fdb_file_size called");

	if size_out.is_null() {
		return SQLITE_IOERR;
	}

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			error!("Null file pointer in fdb_file_size");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		return SQLITE_IOERR;
	}

	// Return the size from metadata
	*size_out = ext.metadata.size;

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_lock(file: *mut sqlite3_file, lock_type: c_int) -> c_int {
	debug!("fdb_file_lock called: lock_type={}", lock_type);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			error!("Null file pointer in fdb_file_lock");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		return SQLITE_IOERR;
	}

	// Convert to our lock state enum
	let requested_lock = LockState::from(lock_type);

	// Simple approach: since we only have a single reader/writer,
	// we can always grant any lock that's requested
	fdb_file.ext.assume_init_mut().lock_state = requested_lock;

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_unlock(file: *mut sqlite3_file, lock_type: c_int) -> c_int {
	debug!("fdb_file_unlock called: lock_type={}", lock_type);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			error!("Null file pointer in fdb_file_unlock");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		return SQLITE_IOERR;
	}

	// Convert to our lock state enum
	let requested_lock = LockState::from(lock_type);

	// Update lock state if downgrading
	if requested_lock as u8 <= ext.lock_state as u8 {
		fdb_file.ext.assume_init_mut().lock_state = requested_lock;
	}

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_check_reserved_lock(
	file: *mut sqlite3_file,
	reserved_out: *mut c_int,
) -> c_int {
	debug!("fdb_file_check_reserved_lock called");

	if reserved_out.is_null() {
		return SQLITE_IOERR;
	}

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			error!("Null file pointer in fdb_file_check_reserved_lock");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		return SQLITE_IOERR;
	}

	// In our simple approach with a single reader/writer, we always
	// report no reserved lock held by others
	*reserved_out = 0;

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_control(
	_file: *mut sqlite3_file,
	op: c_int,
	_arg: *mut c_void,
) -> c_int {
	debug!("fdb_file_control called: op={}", op);

	// We don't support any special file controls
	SQLITE_NOTFOUND
}

pub unsafe extern "C" fn fdb_file_sector_size(_file: *mut sqlite3_file) -> c_int {
	// FoundationDB doesn't have the concept of sectors, so use the SQLite default
	// (usually 512 or 4096 bytes)
	crate::utils::DEFAULT_PAGE_SIZE as c_int
}

pub unsafe extern "C" fn fdb_file_device_characteristics(_file: *mut sqlite3_file) -> c_int {
	// Return supported capabilities
	SQLITE_IOCAP_ATOMIC |             // Writes are atomic 
    SQLITE_IOCAP_SAFE_APPEND |        // Append operations are always consistent
    SQLITE_IOCAP_SEQUENTIAL |         // Sequential writes are efficient 
    SQLITE_IOCAP_UNDELETABLE_WHEN_OPEN // Cannot delete while file is open
}
