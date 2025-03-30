use foundationdb::{Database, FdbBindingError, FdbError};
use libsqlite3_sys::*;
use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_void};
use std::ptr;
use std::slice;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing;

use super::super::fdb::metadata::FdbFileMetadata;
use super::general::FdbVfs;
use crate::metrics;
use crate::impls::pages::utils::{run_fdb_tx, LockState, SQLITE_IOERR, SQLITE_OK};

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
	tracing::debug!("fdb_file_close called");

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_close");
			metrics::record_vfs_error("null_file_pointer_close");
			return SQLITE_IOERR;
		}
	};

	// Extract ext to drop it
	let ext = std::mem::replace(&mut fdb_file.ext, MaybeUninit::uninit());
	let ext_assumed = ext.assume_init();

	// Record metrics for file close
	metrics::record_file_close(true);

	// For clarity, explicitly capture the file path that was closed
	let path = ext_assumed.path;
	tracing::debug!("Successfully closed file: {}", path);

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_read(
	file: *mut sqlite3_file,
	buf: *mut c_void,
	count: c_int,
	offset: i64,
) -> c_int {
	tracing::debug!("fdb_file_read called: count={}, offset={}", count, offset);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_read");
			metrics::record_vfs_error("null_file_pointer_read");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		metrics::record_vfs_error("file_not_open_read");
		return SQLITE_IOERR;
	}

	// Get file path for metrics
	let file_path = &ext.path;

	let vfs = match ext.vfs.as_ref() {
		Some(v) => v,
		None => {
			tracing::error!("Null VFS pointer in fdb_file_read");
			metrics::record_vfs_error("null_vfs_pointer_read");
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
		metrics::record_read_operation(file_path, 0, false);
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
			let mut pages_read = 0;
			let mut partial_pages = 0;

			// Read each page that contains data we need
			for page_num in start_page..=end_page {
				let page_key = keyspace_clone.page_key(&file_id, page_num);

				// Read page data from FoundationDB
				let page_data_result = tx.get(&page_key, false).await?;

				if let Some(page_data) = page_data_result {
					pages_read += 1;

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

					// Determine if this is a partial page read
					let is_partial = relative_start > 0 || bytes_to_copy < page_data.len();
					if is_partial {
						partial_pages += 1;
					}

					// Record page metrics
					metrics::record_page_read(page_data.len(), is_partial);

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
			Ok((bytes_read, data_buffer, pages_read, partial_pages))
		}
	});

	match read_result {
		Ok((bytes_read, data_buffer, _pages_read, _partial_pages)) => {
			// Now copy the data from our buffer to the output buffer
			let buf_slice = slice::from_raw_parts_mut(buf as *mut u8, count as usize);
			buf_slice[..bytes_read].copy_from_slice(&data_buffer[..bytes_read]);

			// Record metrics for the read operation
			let success = bytes_read == count as usize;
			metrics::record_read_operation(file_path, bytes_read, success);

			if bytes_read < count as usize {
				// Short read, not an error but not a full read
				SQLITE_IOERR_SHORT_READ
			} else {
				// Full read
				SQLITE_OK
			}
		}
		Err(e) => {
			tracing::error!("Error reading file: {}", e);
			metrics::record_read_operation(file_path, 0, false);
			metrics::record_vfs_error("read_transaction_error");
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
	tracing::debug!("fdb_file_write called: count={}, offset={}", count, offset);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_write");
			metrics::record_vfs_error("null_file_pointer_write");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		metrics::record_vfs_error("file_not_open_write");
		return SQLITE_IOERR;
	}

	// Get file path for metrics
	let file_path = &ext.path;

	let vfs = match ext.vfs.as_ref() {
		Some(v) => v,
		None => {
			tracing::error!("Null VFS pointer in fdb_file_write");
			metrics::record_vfs_error("null_vfs_pointer_write");
			return SQLITE_IOERR;
		}
	};

	// Copy data from raw pointer to a safe buffer
	let buf_data = unsafe { slice::from_raw_parts(buf as *const u8, count as usize).to_vec() };

	// Extract all needed data before entering the closure
	let file_id = ext.metadata.file_id;
	let page_size = ext.metadata.page_size as i64;
	let current_size = ext.metadata.size;
	let file_path_clone = ext.path.clone();

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
		let file_path_clone = file_path_clone.clone(); // Clone for ownership
		let keyspace_clone = keyspace.clone(); // Clone for ownership
		let buf_data_clone = buf_data.clone(); // Clone for ownership
		let offset = offset; // This is a primitive which is Copy
		let end_offset = end_offset; // This is a primitive which is Copy
		let start_page = start_page; // This is a primitive which is Copy
		let end_page = end_page; // This is a primitive which is Copy

		async move {
			let mut pages_written = 0;
			let mut partial_pages = 0;

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
					pages_written += 1;

					// Calculate buffer indices
					let buf_start = (overlap_start - offset) as usize;
					let buf_end = (overlap_end - offset) as usize;

					// If we're doing a partial page write, we need to read the existing page first
					// and merge our changes with the existing data
					let is_partial = overlap_start > page_offset || overlap_end < page_end;
					if is_partial {
						partial_pages += 1;

						let existing_page_result = tx.get(&page_key, false).await?;

						let mut page_data = match existing_page_result {
							Some(data) => {
								// Use our safe helper to create a properly sized BytesMut
								let mut safe_bytes =
									crate::impls::pages::utils::create_safe_bytes(page_size as usize);
								// Copy existing data if any
								if !data.is_empty() {
									let copy_len = std::cmp::min(data.len(), safe_bytes.len());
									safe_bytes[..copy_len].copy_from_slice(&data[..copy_len]);
								}
								safe_bytes
							}
							None => {
								// New page, initialize with zeros using our safe helper
								crate::impls::pages::utils::create_safe_bytes(page_size as usize)
							}
						};

						// Calculate the offset within the page
						let page_buf_start = (overlap_start - page_offset) as usize;
						let page_buf_end = (overlap_end - page_offset) as usize;

						// Copy the buffer portion to the page data
						page_data[page_buf_start..page_buf_end]
							.copy_from_slice(&buf_data_clone[buf_start..buf_end]);

						// Record page metrics
						metrics::record_page_write(page_data.len(), true);

						// Write the page back to FoundationDB
						tx.set(&page_key, &page_data);
					} else {
						// Full page write, just use the buffer data directly
						let page_data = &buf_data_clone[buf_start..buf_end];

						// Record page metrics
						metrics::record_page_write(page_data.len(), false);

						tx.set(&page_key, page_data);
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

				// Start metrics timer for metadata operation
				let metadata_timer = metrics::record_metadata_read();

				// Get existing metadata to preserve creation time and update modification time
				let metadata_key = keyspace_clone.metadata_key(&file_path_clone);
				let metadata_result = tx.get(&metadata_key, false).await?;

				// Complete metrics
				metrics::complete_metadata_read(&metadata_timer);

				if let Some(metadata_bytes_vec) = metadata_result {
					let metadata_bytes = &metadata_bytes_vec;
					if let Ok(old_metadata) = FdbFileMetadata::from_bytes(&metadata_bytes) {
						updated_metadata.created_at = old_metadata.created_at;
						updated_metadata.modified_at = if let Some(new_time) = updated_modified_at {
							new_time
						} else {
							old_metadata.modified_at
						};
					}
				}

				// Start metrics timer for metadata write
				let metadata_write_timer = metrics::record_metadata_write();

				// Write updated metadata to FoundationDB
				let metadata_bytes = updated_metadata.to_bytes();
				tx.set(&metadata_key, &metadata_bytes);

				// Complete metrics
				metrics::complete_metadata_write(&metadata_write_timer);
			}

			Ok((new_size, pages_written, partial_pages))
		}
	});

	match write_result {
		Ok((new_size, _pages_written, _partial_pages)) => {
			// Store file_path for metrics before we mutate the struct
			let path_for_metrics = file_path.to_string();
			let count_for_metrics = count as usize;

			// Update the in-memory file size
			(*fdb_file).ext.assume_init_mut().metadata.size = new_size;

			// Record metrics for the write operation
			metrics::record_write_operation(&path_for_metrics, count_for_metrics, true);

			// Update file size metrics
			metrics::update_file_size(&path_for_metrics, new_size);

			SQLITE_OK
		}
		Err(e) => {
			tracing::error!("Error writing file: {}", e);

			// Record metrics for failed write operation
			metrics::record_write_operation(file_path, 0, false);
			metrics::record_vfs_error("write_transaction_error");

			SQLITE_IOERR
		}
	}
}

pub unsafe extern "C" fn fdb_file_truncate(file: *mut sqlite3_file, size: i64) -> c_int {
	tracing::debug!("fdb_file_truncate called: size={}", size);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_truncate");
			metrics::record_vfs_error("null_file_pointer_truncate");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		metrics::record_vfs_error("file_not_open_truncate");
		return SQLITE_IOERR;
	}

	// Get file path for metrics
	let file_path = &ext.path;

	let vfs = match ext.vfs.as_ref() {
		Some(v) => v,
		None => {
			tracing::error!("Null VFS pointer in fdb_file_truncate");
			metrics::record_vfs_error("null_vfs_pointer_truncate");
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
	let file_path_clone = ext.path.clone();

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
		let file_path_clone = file_path_clone.clone(); // Clone for ownership
		let keyspace_clone = keyspace.clone(); // Clone for ownership
		let pages_to_keep = pages_to_keep; // This is a primitive which is Copy
		let pages_to_delete = pages_to_delete; // This is a primitive which is Copy
		let size = size; // This is a primitive which is Copy

		async move {
			// Start metrics timer for metadata read
			let metadata_timer = metrics::record_metadata_read();

			// Get existing metadata
			let metadata_key = keyspace_clone.metadata_key(&file_path_clone);
			let metadata_result = tx.get(&metadata_key, false).await?;

			// Complete metrics
			metrics::complete_metadata_read(&metadata_timer);

			// Create updated metadata to save
			let updated_metadata = if let Some(metadata_bytes_vec) = metadata_result {
				let metadata_bytes = &metadata_bytes_vec;
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
						metrics::record_vfs_error("metadata_parse_error_truncate");
						return Err(FdbBindingError::NonRetryableFdbError(FdbError::from_code(
							1,
						)));
					}
				}
			} else {
				// This shouldn't happen but handle it gracefully
				metrics::record_vfs_error("missing_metadata_truncate");
				return Err(FdbBindingError::NonRetryableFdbError(FdbError::from_code(
					1,
				)));
			};

			// Start metrics timer for metadata write
			let metadata_write_timer = metrics::record_metadata_write();

			// Save updated metadata
			let metadata_bytes = updated_metadata.to_bytes();
			tx.set(&metadata_key, &metadata_bytes);

			// Complete metrics
			metrics::complete_metadata_write(&metadata_write_timer);

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
			// Store file_path for metrics before we mutate the struct
			let path_for_metrics = file_path.to_string();

			// Update the in-memory metadata
			(*fdb_file).ext.assume_init_mut().metadata = updated_metadata;

			// Record metrics for file truncate operation
			metrics::record_file_truncate(&path_for_metrics, size, true);

			// Update file size metrics
			metrics::update_file_size(&path_for_metrics, size);

			SQLITE_OK
		}
		Err(e) => {
			tracing::error!("Error truncating file: {}", e);

			// Record metrics for failed truncate operation
			metrics::record_file_truncate(file_path, size, false);
			metrics::record_vfs_error("truncate_error");

			SQLITE_IOERR
		}
	}
}

pub unsafe extern "C" fn fdb_file_sync(_file: *mut sqlite3_file, _flags: c_int) -> c_int {
	// FoundationDB is always synced, so we don't need to do anything
	tracing::debug!("fdb_file_sync called");
	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_size(file: *mut sqlite3_file, size_out: *mut i64) -> c_int {
	tracing::debug!("fdb_file_size called");

	if size_out.is_null() {
		return SQLITE_IOERR;
	}

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_size");
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
	tracing::debug!("fdb_file_lock called: lock_type={}", lock_type);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_lock");
			metrics::record_vfs_error("null_file_pointer_lock");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		metrics::record_vfs_error("file_not_open_lock");
		return SQLITE_IOERR;
	}

	// Convert to our lock state enum
	let requested_lock = LockState::from(lock_type);
	let current_lock = ext.lock_state;

	// Record lock acquisition
	match requested_lock {
		LockState::None => metrics::record_lock_acquisition("none"),
		LockState::Shared => metrics::record_lock_acquisition("shared"),
		LockState::Reserved => metrics::record_lock_acquisition("reserved"),
		LockState::Pending => metrics::record_lock_acquisition("pending"),
		LockState::Exclusive => metrics::record_lock_acquisition("exclusive"),
	}

	// Check if this is a lock escalation
	if (requested_lock as u8) > (current_lock as u8) {
		metrics::record_lock_escalation();
	}

	// Simple approach: since we only have a single reader/writer,
	// we can always grant any lock that's requested
	fdb_file.ext.assume_init_mut().lock_state = requested_lock;

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_unlock(file: *mut sqlite3_file, lock_type: c_int) -> c_int {
	tracing::debug!("fdb_file_unlock called: lock_type={}", lock_type);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_unlock");
			metrics::record_vfs_error("null_file_pointer_unlock");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		metrics::record_vfs_error("file_not_open_unlock");
		return SQLITE_IOERR;
	}

	// Convert to our lock state enum
	let requested_lock = LockState::from(lock_type);

	// Record lock acquisition (downgrade)
	match requested_lock {
		LockState::None => metrics::record_lock_acquisition("none"),
		LockState::Shared => metrics::record_lock_acquisition("shared"),
		LockState::Reserved => metrics::record_lock_acquisition("reserved"),
		LockState::Pending => metrics::record_lock_acquisition("pending"),
		LockState::Exclusive => metrics::record_lock_acquisition("exclusive"),
	}

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
	tracing::debug!("fdb_file_check_reserved_lock called");

	if reserved_out.is_null() {
		return SQLITE_IOERR;
	}

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_check_reserved_lock");
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
	tracing::debug!("fdb_file_control called: op={}", op);

	// We don't support any special file controls
	SQLITE_NOTFOUND
}

pub unsafe extern "C" fn fdb_file_sector_size(_file: *mut sqlite3_file) -> c_int {
	// FoundationDB doesn't have the concept of sectors, so use the SQLite default
	// (usually 512 or 4096 bytes)
	crate::impls::pages::utils::DEFAULT_PAGE_SIZE as c_int
}

pub unsafe extern "C" fn fdb_file_device_characteristics(_file: *mut sqlite3_file) -> c_int {
	// Return supported capabilities
	SQLITE_IOCAP_ATOMIC |             // Writes are atomic 
    SQLITE_IOCAP_SAFE_APPEND |        // Append operations are always consistent
    SQLITE_IOCAP_SEQUENTIAL |         // Sequential writes are efficient 
    SQLITE_IOCAP_UNDELETABLE_WHEN_OPEN // Cannot delete while file is open
}
