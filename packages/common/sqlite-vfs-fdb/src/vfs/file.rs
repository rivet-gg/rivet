use foundationdb::Database;
use libsqlite3_sys::*;
use std::cmp::min;
use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_void};
use std::ptr;
use std::slice;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing;
use uuid::Uuid;

use super::super::fdb::metadata::FdbFileMetadata;
use super::general::FdbVfs;
use crate::metrics;
use crate::utils::{run_fdb_tx, FdbVfsError, LockState, SQLITE_IOERR, SQLITE_OK};

// Constants for SHM implementation
pub const SHM_INITIAL_SIZE: usize = 32768; // 32KB initial size
pub const SHM_REGION_SIZE: usize = 8192; // 8KB per region (typical SQLite default)
const MAX_SHM_REGIONS: usize = 10; // Maximum number of regions

/// Enum representing the type of SQLite file
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SqliteFileType {
	/// Main database file (.db)
	Database,
	/// Write-Ahead Log file (.db-wal)
	WAL,
}

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
	/// Type of SQLite file
	pub file_type: SqliteFileType,
	/// SHM memory buffer (only used for SHM files)
	pub shm_buffer: Option<Vec<u8>>,
	/// SHM region size (only used for SHM files)
	pub shm_region_size: usize,
	/// Number of active SHM maps (for reference counting)
	pub shm_map_count: u32,
}

/// The I/O methods for FdbFile
pub static FDB_IO_METHODS: sqlite3_io_methods = sqlite3_io_methods {
	iVersion: 3, // Updated to version 3 to support SHM functions
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
	xShmMap: Some(fdb_shm_map),
	xShmLock: Some(fdb_shm_lock),
	xShmBarrier: Some(fdb_shm_barrier),
	xShmUnmap: Some(fdb_shm_unmap),
	xFetch: None,
	xUnfetch: None,
};

/// Get file pointer safely
unsafe fn get_file_ptr<'a>(file: *mut FdbFile, context: &str) -> Result<&'a mut FdbFile, c_int> {
	match (file as *mut FdbFile).as_mut() {
		Some(f) => Ok(f),
		None => {
			tracing::error!("Null file pointer in {}", context);
			metrics::record_vfs_error(&format!("null_file_pointer_{}", context));
			Err(SQLITE_IOERR)
		}
	}
}

/// Get file extension reference safely
unsafe fn get_file_extension<'a>(
	file: *mut FdbFile,
	context: &str,
) -> Result<&'a FdbFileExt, c_int> {
	let fdb_file = get_file_ptr(file, context)?;

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		metrics::record_vfs_error(&format!("file_not_open_{}", context));
		return Err(SQLITE_IOERR);
	}

	Ok(ext)
}

/// Get file extension reference mutably and safely
unsafe fn get_file_extension_mut<'a>(
	file: *mut FdbFile,
	context: &str,
) -> Result<&'a mut FdbFileExt, c_int> {
	let fdb_file = get_file_ptr(file, context)?;

	let ext = fdb_file.ext.assume_init_mut();
	if !ext.is_open {
		metrics::record_vfs_error(&format!("file_not_open_{}", context));
		return Err(SQLITE_IOERR);
	}

	Ok(ext)
}

/// Get VFS instance from extension
unsafe fn get_vfs_from_ext<'a>(ext: &'a FdbFileExt, context: &str) -> Result<&'a FdbVfs, c_int> {
	match ext.vfs.as_ref() {
		Some(vfs) => Ok(vfs),
		None => {
			tracing::error!("Null VFS pointer in {}", context);
			metrics::record_vfs_error(&format!("null_vfs_pointer_{}", context));
			Err(SQLITE_IOERR)
		}
	}
}

/// Read pages from FoundationDB
fn read_pages_from_fdb(
	vfs: &FdbVfs,
	file_id: Uuid,
	offset: i64,
	count: usize,
	page_size: i64,
	file_size: i64,
) -> Result<(Vec<u8>, usize), FdbVfsError> {
	tracing::info!(
		"read_pages_from_fdb: file_id={}, offset={}, count={}, page_size={}, file_size={}",
		file_id,
		offset,
		count,
		page_size,
		file_size
	);

	// Create a buffer to store the data we read, filled with zeros
	let mut data_buffer = vec![0u8; count];
	let keyspace = vfs.keyspace.clone();
	let db = vfs.db.clone();

	// Handle the case where we're reading a brand new file with size 0
	if file_size == 0 {
		tracing::info!("File size is 0, returning zero-filled buffer");
		return Ok((data_buffer, 0));
	}

	// Calculate the range of pages to read
	let start_page = (offset / page_size) as u32;
	let end_offset = min(offset + (count as i64), file_size);
	let end_page = ((end_offset - 1) / page_size) as u32;

	tracing::info!(
		"Reading pages: start_page={}, end_page={}, end_offset={}",
		start_page,
		end_page,
		end_offset
	);

	// If we're reading beyond the file size, return zeros for that portion
	if offset >= file_size {
		tracing::info!("Reading past EOF, returning zero-filled buffer");
		return Ok((data_buffer, 0));
	}

	let mut bytes_read = 0;

	// Read each page that contains data we need, using separate transactions per page
	// to avoid transaction conflicts
	for page_num in start_page..=end_page {
		let page_key = keyspace.page_key(&file_id, page_num);
		tracing::info!("Reading page {} for file {}", page_num, file_id);

		// Use a separate transaction for each page to improve reliability
		let page_result = run_fdb_tx(&db, move |tx| {
			let page_key = page_key.clone();

			async move {
				match tx.get(&page_key, false).await {
					Ok(result) => Ok(result),
					Err(e) => Err(foundationdb::FdbBindingError::from(e)),
				}
			}
		});

		match page_result {
			Ok(Some(page_data)) => {
				tracing::info!("  Found page {}, length={}", page_num, page_data.len());

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

				let bytes_to_copy = min(page_bytes_available, count - bytes_read);

				// Determine if this is a partial page read
				let is_partial = relative_start > 0 || bytes_to_copy < page_data.len();

				// Record page metrics
				metrics::record_page_read(page_data.len(), is_partial);

				if bytes_to_copy > 0 {
					// Copy data to our buffer
					let dest_start = if page_offset < offset {
						0
					} else {
						(page_offset - offset) as usize
					};

					tracing::info!(
						"  Copying bytes: relative_start={}, bytes_to_copy={}, dest_start={}",
						relative_start,
						bytes_to_copy,
						dest_start
					);

					if dest_start + bytes_to_copy <= data_buffer.len() {
						data_buffer[dest_start..dest_start + bytes_to_copy].copy_from_slice(
							&page_data[relative_start..relative_start + bytes_to_copy],
						);

						bytes_read += bytes_to_copy;
					} else {
						tracing::error!(
							"  Buffer overflow: dest_start={}, bytes_to_copy={}, buffer_len={}",
							dest_start,
							bytes_to_copy,
							data_buffer.len()
						);
					}
				}
			}
			Ok(None) => {
				tracing::info!("  Page {} not found", page_num);
			}
			Err(e) => {
				tracing::error!("  Error reading page {}: {}", page_num, e);
				// Continue with other pages instead of failing completely
			}
		}
	}

	tracing::info!("Total bytes read: {}", bytes_read);

	// If we read any data, return success
	if bytes_read > 0 || end_offset == 0 {
		tracing::info!("read_pages_from_fdb completed successfully");
		Ok((data_buffer, bytes_read))
	} else {
		// If we should have read data but didn't, return an error
		tracing::error!(
			"Failed to read any data when expected file_size={}",
			file_size
		);
		Err(FdbVfsError::Other(
			"Failed to read any data from file".to_string(),
		))
	}
}

pub unsafe extern "C" fn fdb_file_read(
	file: *mut sqlite3_file,
	buf: *mut c_void,
	count: c_int,
	offset: i64,
) -> c_int {
	tracing::info!("fdb_file_read called: count={}, offset={}", count, offset);

	// Get file extension safely
	let ext = match get_file_extension(file as *mut FdbFile, "fdb_file_read") {
		Ok(ext) => ext,
		Err(code) => {
			tracing::error!("Failed to get file extension in fdb_file_read: {}", code);
			return code;
		}
	};

	// Get file path for metrics
	let file_path = &ext.path;
	tracing::info!(
		"Reading from file: {}, file_type: {:?}",
		file_path,
		ext.file_type
	);

	// Get VFS reference
	let vfs = match get_vfs_from_ext(ext, "fdb_file_read") {
		Ok(vfs) => vfs,
		Err(code) => {
			tracing::error!("Failed to get VFS from ext in fdb_file_read: {}", code);
			return code;
		}
	};

	// Zero the output buffer by default - this ensures we don't return uninitialized memory
	// This is important for reads beyond EOF or for short reads
	ptr::write_bytes(buf, 0, count as usize);

	// Handle read based on file type
	match ext.file_type {
		SqliteFileType::WAL => {
			tracing::info!("Reading from WAL file: {}", file_path);

			// Create WAL manager
			let wal_manager = crate::wal::WalManager::new(ext.db.clone(), vfs.keyspace.clone());

			// Use WAL manager to read data
			match wal_manager.read_wal_data(&ext.metadata.file_id, offset, count as usize) {
				Ok(data) => {
					// We already zeroed the buffer, so now just copy the actual data
					if !data.is_empty() {
						let buf_slice = slice::from_raw_parts_mut(buf as *mut u8, count as usize);
						buf_slice[..data.len()].copy_from_slice(&data);
					}

					// Record metrics
					metrics::record_read_operation(
						file_path,
						data.len(),
						data.len() == count as usize,
					);

					tracing::info!("WAL read success: read {} bytes", data.len());

					// For WAL files, we return SQLITE_OK even for short reads
					// This is because WAL frames might be smaller than requested
					SQLITE_OK
				}
				Err(e) => {
					tracing::error!("Error reading WAL file: {}", e);
					metrics::record_read_operation(file_path, 0, false);
					metrics::record_vfs_error("wal_read_error");
					SQLITE_IOERR
				}
			}
		}
		SqliteFileType::Database => {
			tracing::info!("Reading from Database file: {}", file_path);
			// Extract data needed for reading
			let file_id = ext.metadata.file_id;
			let page_size = ext.metadata.page_size as i64;
			let file_size = ext.metadata.size;

			tracing::info!(
				"Reading regular file: file_id={}, offset={}, count={}, page_size={}, file_size={}",
				file_id,
				offset,
				count,
				page_size,
				file_size
			);

			// Handle read beyond EOF as a short read, not an error
			if offset >= file_size {
				// Buffer is already zeroed, just record metrics and return
				metrics::record_read_operation(file_path, 0, false);
				tracing::info!("Read beyond EOF, returning SQLITE_OK with zero bytes");
				// SQLite expects SQLITE_OK for reads past the end, with a zero buffer
				return SQLITE_OK;
			}

			// Determine how much we can actually read
			let expected_read_size = min(count as i64, file_size - offset) as usize;
			if expected_read_size == 0 {
				// Edge case: file exists but we're reading at exactly the file size
				metrics::record_read_operation(file_path, 0, true);
				return SQLITE_OK;
			}

			// Read data from FoundationDB
			let read_result =
				read_pages_from_fdb(vfs, file_id, offset, count as usize, page_size, file_size);

			match read_result {
				Ok((data_buffer, bytes_read)) => {
					// Copy the data from our buffer to the output buffer
					// The output buffer is already zeroed, so we just need to copy the actual data
					if bytes_read > 0 {
						let buf_slice = slice::from_raw_parts_mut(buf as *mut u8, count as usize);
						buf_slice[..bytes_read].copy_from_slice(&data_buffer[..bytes_read]);
					}

					// Record metrics for the read operation
					let success = bytes_read == expected_read_size;
					metrics::record_read_operation(file_path, bytes_read, success);

					if bytes_read == 0 && expected_read_size > 0 {
						// We expected to read data but got nothing - this is an error
						tracing::error!(
							"Expected to read {} bytes but got none",
							expected_read_size
						);
						// This indicates a real I/O error, not just a short read
						SQLITE_IOERR
					} else if bytes_read < expected_read_size {
						// Short read - we got some data but not all we expected
						tracing::warn!(
							"Short read: read {} bytes, expected {} bytes",
							bytes_read,
							expected_read_size
						);
						// For SQLite, a short read is a specific error code
						SQLITE_IOERR_SHORT_READ
					} else {
						// Full read - we got all the data we expected
						tracing::info!("Full read success: read {} bytes", bytes_read);
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
	}
}

pub unsafe extern "C" fn fdb_file_write(
	file: *mut sqlite3_file,
	buf: *const c_void,
	count: c_int,
	offset: i64,
) -> c_int {
	tracing::info!("fdb_file_write called: count={}, offset={}", count, offset);

	// Get file information
	let fdb_file = match get_file_ptr(file as *mut FdbFile, "fdb_file_write") {
		Ok(f) => f,
		Err(e) => {
			tracing::error!("Failed to get file pointer in fdb_file_write: {}", e);
			return e;
		}
	};

	// First use an immutable reference to get all the data we need
	let ext_ref = fdb_file.ext.assume_init_ref();
	let file_type = ext_ref.file_type;
	let file_path = ext_ref.path.clone();
	let file_id = ext_ref.metadata.file_id;
	let page_size = ext_ref.metadata.page_size as i64;
	let current_size = ext_ref.metadata.size;

	// Get VFS information
	let vfs = match get_vfs_from_ext(ext_ref, "fdb_file_write") {
		Ok(v) => v,
		Err(code) => {
			tracing::error!("Failed to get VFS from ext in fdb_file_write: {}", code);
			return code;
		}
	};

	// Get clones of VFS data for transactions
	let db = vfs.db.clone();
	let keyspace = vfs.keyspace.clone();

	tracing::info!("Writing to file: {}, file_type: {:?}", file_path, file_type);

	// Copy data from raw pointer to a safe buffer
	let buf_data = slice::from_raw_parts(buf as *const u8, count as usize).to_vec();

	// Print a preview of the first 32 bytes of data or less
	let preview_size = std::cmp::min(32, buf_data.len());
	let preview_bytes = &buf_data[..preview_size];
	tracing::info!(
		"Write data preview (first {} bytes): {:?}",
		preview_size,
		preview_bytes
	);

	// Calculate new file size
	let end_offset = offset + (count as i64);

	// Update in-memory size immediately if needed
	if end_offset > current_size {
		// Now use a mutable reference to update the metadata
		let ext_mut = fdb_file.ext.assume_init_mut();
		tracing::info!("Updating file size from {} to {}", current_size, end_offset);
		ext_mut.metadata.size = end_offset;
	}

	// Dispatch to appropriate write handler based on file type
	match file_type {
		SqliteFileType::WAL => write_wal_file(
			&file_path,
			file_id,
			offset,
			&buf_data,
			page_size as usize,
			&db,
			&keyspace,
		),
		SqliteFileType::Database => {
			// For database and SHM files
			write_regular_file(
				&file_path,
				file_id,
				offset,
				count,
				&buf_data,
				page_size,
				current_size,
				end_offset,
				&db,
				&keyspace,
			)
		}
	}
}

// Helper function to write WAL file
fn write_wal_file(
	file_path: &str,
	file_id: Uuid,
	offset: i64,
	buf_data: &[u8],
	page_size: usize,
	db: &Arc<Database>,
	keyspace: &crate::fdb::keyspace::FdbKeySpace,
) -> c_int {
	tracing::debug!("Writing to WAL file");

	// Create WAL manager
	let wal_manager = crate::wal::WalManager::new(db.clone(), keyspace.clone());

	// Process the WAL write
	match wal_manager.process_wal_write(&file_id, offset, &buf_data, page_size) {
		Ok(bytes_written) => {
			metrics::record_write_operation(file_path, bytes_written, 1, 0);
			SQLITE_OK
		}
		Err(e) => {
			tracing::error!("Error writing WAL file: {}", e);
			metrics::record_vfs_error("wal_write_error");
			SQLITE_IOERR
		}
	}
}

// Helper function to write regular file (database or SHM)
fn write_regular_file(
	file_path: &str,
	file_id: Uuid,
	offset: i64,
	count: c_int,
	buf_data: &[u8],
	page_size: i64,
	current_size: i64,
	end_offset: i64,
	db: &Arc<Database>,
	keyspace: &crate::fdb::keyspace::FdbKeySpace,
) -> c_int {
	// Clone the keyspace for use in closures
	let keyspace_cloned = keyspace.clone();
	// Calculate the range of pages to write
	let start_page = (offset / page_size) as u32;
	let end_page = ((end_offset - 1) / page_size) as u32;

	tracing::info!(
		"Writing regular file: file_id={}, offset={}, count={}, start_page={}, end_page={}",
		file_id,
		offset,
		count,
		start_page,
		end_page
	);

	// Track success and statistics
	let mut write_success = true;
	let mut pages_written = 0;
	let mut partial_pages = 0;

	// Write each page in its own transaction
	for page_num in start_page..=end_page {
		let page_key = keyspace_cloned.page_key(&file_id, page_num);
		let page_offset = page_num as i64 * page_size;
		let page_end = page_offset + page_size;

		// Calculate the overlap between the buffer and this page
		let overlap_start = std::cmp::max(offset, page_offset);
		let overlap_end = std::cmp::min(end_offset, page_end);

		if overlap_start >= overlap_end {
			continue; // No overlap with this page
		}

		pages_written += 1;

		// Calculate buffer indices
		let buf_start = (overlap_start - offset) as usize;
		let buf_end = (overlap_end - offset) as usize;
		let buf_page_data = buf_data[buf_start..buf_end].to_vec();

		// Determine if this is a partial page write
		let is_partial = overlap_start > page_offset || overlap_end < page_end;

		// Write this page
		let page_result = if is_partial {
			partial_pages += 1;
			// For partial writes, we need to read-modify-write
			let page_write_result = run_fdb_tx(db, move |tx| {
				let page_key = page_key.clone();
				let buf_page_data = buf_page_data.clone();
				let page_size = page_size;
				let overlap_start = overlap_start;
				let page_offset = page_offset;

				async move {
					// Read existing page data if any
					let existing_page_result = tx
						.get(&page_key, false)
						.await
						.map_err(|e| foundationdb::FdbBindingError::from(e))?;

					let mut page_data = match existing_page_result {
						Some(data) => {
							// Use our safe helper to create a properly sized BytesMut
							let mut safe_bytes =
								crate::utils::create_safe_bytes(page_size as usize);
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
					let page_buf_end = page_buf_start + buf_page_data.len();

					// Copy the buffer portion to the page data
					page_data[page_buf_start..page_buf_end].copy_from_slice(&buf_page_data);

					// Record page metrics
					metrics::record_page_write(page_data.len(), true);

					// Write the page back to FoundationDB
					tx.set(&page_key, &page_data);

					Ok(())
				}
			});

			page_write_result
		} else {
			// Full page write, just use the buffer data directly
			let page_write_result = run_fdb_tx(db, move |tx| {
				let page_key = page_key.clone();
				let buf_page_data = buf_page_data.clone();

				async move {
					// Record page metrics
					metrics::record_page_write(buf_page_data.len(), false);

					// Write the page to FoundationDB
					tx.set(&page_key, &buf_page_data);

					Ok(())
				}
			});

			page_write_result
		};

		if let Err(e) = page_result {
			tracing::error!("Error writing page {}: {}", page_num, e);
			write_success = false;
			break;
		}
	}

	// If all page writes were successful, update metadata if needed
	if write_success && end_offset > current_size {
		// Update the file size in metadata
		let file_path_copy = file_path.to_string();
		// We need to clone again since keyspace_cloned will be used in verification later
		let keyspace_cloned_for_metadata = keyspace_cloned.clone();
		let metadata_update_result = run_fdb_tx(db, move |tx| {
			let file_path = file_path_copy.clone();
			let keyspace_clone = keyspace_cloned_for_metadata.clone();

			async move {
				// Create new metadata with the updated size
				let mut updated_metadata = FdbFileMetadata {
					file_id,
					size: end_offset,
					page_size: page_size as usize,
					// Placeholder values, will be updated from metadata
					created_at: 0,
					modified_at: SystemTime::now()
						.duration_since(UNIX_EPOCH)
						.unwrap_or_default()
						.as_secs(),
				};

				// Get current metadata to preserve created_at
				let metadata_key = keyspace_clone.metadata_key(&file_path);
				let current_metadata_result = tx
					.get(&metadata_key, false)
					.await
					.map_err(|e| foundationdb::FdbBindingError::from(e))?;

				// Parse the current metadata to get the created_at time
				if let Some(metadata_bytes) = current_metadata_result {
					if let Ok(current_metadata) = FdbFileMetadata::from_bytes(&metadata_bytes) {
						updated_metadata.created_at = current_metadata.created_at;
					}
				}

				// Write updated metadata back to FoundationDB
				let metadata_bytes = updated_metadata.to_bytes();
				tx.set(&metadata_key, &metadata_bytes);

				Ok(())
			}
		});

		if let Err(e) = metadata_update_result {
			tracing::error!("Error updating metadata: {}", e);
			write_success = false;
		}
	}

	// Record write metrics
	metrics::record_write_operation(file_path, buf_data.len(), pages_written, partial_pages);

	if write_success {
		// Verify that at least the first page is readable to ensure consistency
		if pages_written > 0 {
			let verify_page_num = start_page;
			let verify_key = keyspace_cloned.page_key(&file_id, verify_page_num);

			let verify_result = run_fdb_tx(db, move |tx| {
				let verify_key = verify_key.clone();

				async move {
					match tx.get(&verify_key, false).await {
						Ok(result) => Ok(result),
						Err(e) => Err(foundationdb::FdbBindingError::from(e)),
					}
				}
			});

			match verify_result {
				Ok(Some(_)) => {
					tracing::info!("Write verification successful for page {}", verify_page_num);
					SQLITE_OK
				}
				Ok(None) => {
					tracing::error!(
						"Write verification failed: page {} was not found after writing",
						verify_page_num
					);
					metrics::record_vfs_error("write_verification_failed");
					SQLITE_IOERR
				}
				Err(e) => {
					tracing::error!("Error during write verification: {}", e);
					metrics::record_vfs_error("write_verification_error");
					SQLITE_IOERR
				}
			}
		} else {
			// No pages were written (edge case)
			SQLITE_OK
		}
	} else {
		tracing::error!("Error during file write operation");
		metrics::record_vfs_error("write_transaction_error");
		SQLITE_IOERR
	}
}

pub unsafe extern "C" fn fdb_file_truncate(file: *mut sqlite3_file, size: i64) -> c_int {
	tracing::debug!("fdb_file_truncate called: size={}", size);

	let fdb_file = match get_file_ptr(file as *mut FdbFile, "fdb_file_truncate") {
		Ok(f) => f,
		Err(e) => return e,
	};

	let ext = fdb_file.ext.assume_init_ref();
	let vfs = match get_vfs_from_ext(ext, "fdb_file_truncate") {
		Ok(vfs) => vfs,
		Err(code) => return code,
	};

	// Get existing file metadata
	let mut metadata = ext.metadata.clone();
	let file_path = ext.path.clone();

	// If new size is same as current, nothing to do
	if metadata.size == size {
		return SQLITE_OK;
	}

	// Update the file size
	metadata.size = size;
	metadata.modified_at = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap_or_default()
		.as_secs();

	// Store the updated metadata
	match vfs.store_metadata(&file_path, &metadata) {
		Ok(_) => SQLITE_OK,
		Err(e) => {
			tracing::error!("Error truncating file: {}", e);
			metrics::record_vfs_error("truncate_transaction_error");
			SQLITE_IOERR
		}
	}
}

pub unsafe extern "C" fn fdb_file_sync(_: *mut sqlite3_file, _: c_int) -> c_int {
	// In FoundationDB, all writes are atomic and durable upon transaction commit
	// We don't need to do anything for sync
	tracing::debug!("fdb_file_sync called (no-op)");
	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_size(file: *mut sqlite3_file, size_out: *mut i64) -> c_int {
	tracing::debug!("fdb_file_size called");

	let ext = match get_file_extension(file as *mut FdbFile, "fdb_file_size") {
		Ok(ext) => ext,
		Err(code) => return code,
	};

	// Copy the size from metadata to the output parameter
	*size_out = ext.metadata.size;
	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_lock(file: *mut sqlite3_file, lock_type: c_int) -> c_int {
	tracing::debug!("fdb_file_lock called: lock_type={}", lock_type);

	let fdb_file = match get_file_ptr(file as *mut FdbFile, "fdb_file_lock") {
		Ok(f) => f,
		Err(e) => return e,
	};

	let ext = fdb_file.ext.assume_init_mut();

	// Convert lock_type to LockState
	let requested_lock = match lock_type {
		0 => LockState::None,
		1 => LockState::Shared,
		2 => LockState::Reserved,
		3 => LockState::Pending,
		4 => LockState::Exclusive,
		_ => {
			tracing::error!("Invalid lock type: {}", lock_type);
			metrics::record_vfs_error("invalid_lock_type");
			return SQLITE_IOERR;
		}
	};

	// In our simplified implementation for a single reader/writer,
	// we always grant the lock
	ext.lock_state = requested_lock;
	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_unlock(file: *mut sqlite3_file, lock_type: c_int) -> c_int {
	tracing::debug!("fdb_file_unlock called: lock_type={}", lock_type);

	let fdb_file = match get_file_ptr(file as *mut FdbFile, "fdb_file_unlock") {
		Ok(f) => f,
		Err(e) => return e,
	};

	let ext = fdb_file.ext.assume_init_mut();

	// Convert lock_type to LockState
	let requested_lock = match lock_type {
		0 => LockState::None,
		1 => LockState::Shared,
		2 => LockState::Reserved,
		3 => LockState::Pending,
		4 => LockState::Exclusive,
		_ => {
			tracing::error!("Invalid lock type: {}", lock_type);
			metrics::record_vfs_error("invalid_lock_type");
			return SQLITE_IOERR;
		}
	};

	// Only downgrade the lock if the requested lock is lower than the current one
	if requested_lock as u8 <= ext.lock_state as u8 {
		ext.lock_state = requested_lock;
	}

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_check_reserved_lock(
	file: *mut sqlite3_file,
	result_out: *mut c_int,
) -> c_int {
	tracing::debug!("fdb_file_check_reserved_lock called");

	let ext = match get_file_extension(file as *mut FdbFile, "fdb_file_check_reserved_lock") {
		Ok(ext) => ext,
		Err(code) => return code,
	};

	// For single reader/writer, we can simplify this
	// The result is 1 if this connection has a reserved, pending, or exclusive lock
	// and 0 otherwise
	*result_out = if ext.lock_state >= LockState::Reserved {
		1
	} else {
		0
	};

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_control(
	file: *mut sqlite3_file,
	op: c_int,
	arg: *mut c_void,
) -> c_int {
	// Handle various file control operations
	tracing::debug!("fdb_file_control called: op={}", op);

	// We need to handle all the various file control operations from SQLite
	// Reference: https://sqlite.org/c3ref/c_fcntl_begin_atomic_write.html
	match op {
		// File size hint
		libsqlite3_sys::SQLITE_FCNTL_SIZE_HINT => {
			// SQLite is giving us a hint about the final file size
			// We can ignore this as we resize on demand
			if !arg.is_null() {
				tracing::debug!("Got size hint: {}", *(arg as *mut i64));
			}
			SQLITE_OK
		}

		// Chunk size for allocation
		libsqlite3_sys::SQLITE_FCNTL_CHUNK_SIZE => {
			// SQLite is suggesting a chunk size for allocation
			// We can ignore this as we use page-based storage
			if !arg.is_null() {
				tracing::debug!("Got chunk size suggestion: {}", *(arg as *mut c_int));
			}
			SQLITE_OK
		}

		// File system specific commands
		libsqlite3_sys::SQLITE_FCNTL_LOCKSTATE
		| libsqlite3_sys::SQLITE_FCNTL_GET_LOCKPROXYFILE
		| libsqlite3_sys::SQLITE_FCNTL_SET_LOCKPROXYFILE
		| libsqlite3_sys::SQLITE_FCNTL_LAST_ERRNO => {
			// We don't need to implement these for our VFS
			tracing::debug!("Ignoring file system specific control op: {}", op);
			SQLITE_OK
		}

		// Atomic write operations
		libsqlite3_sys::SQLITE_FCNTL_BEGIN_ATOMIC_WRITE => {
			// Begin an atomic write transaction
			// For FoundationDB VFS, all writes are already atomic, so we can just acknowledge
			tracing::debug!("Begin atomic write operation (no-op for FoundationDB)");
			SQLITE_OK
		}

		libsqlite3_sys::SQLITE_FCNTL_COMMIT_ATOMIC_WRITE => {
			// Commit an atomic write transaction
			// For FoundationDB VFS, all writes are already atomic within transactions
			tracing::debug!("Commit atomic write operation (no-op for FoundationDB)");
			SQLITE_OK
		}

		libsqlite3_sys::SQLITE_FCNTL_ROLLBACK_ATOMIC_WRITE => {
			// Rollback an atomic write transaction
			// For FoundationDB VFS, we handle transaction atomicity at the FDB level
			tracing::debug!("Rollback atomic write operation (no-op for FoundationDB)");
			SQLITE_OK
		}

		// Commit phase control operations
		libsqlite3_sys::SQLITE_FCNTL_COMMIT_PHASETWO => {
			// We don't need special atomic write support since FoundationDB
			// handles atomicity for us
			tracing::debug!("Commit phase control op: {}", op);
			SQLITE_OK
		}

		// VFS feature detection
		libsqlite3_sys::SQLITE_FCNTL_VFSNAME => {
			// Return the name of our VFS
			if !arg.is_null() {
				let vfs_name_ptr = arg as *mut *mut i8;
				let c_str = std::ffi::CString::new(crate::utils::FDB_VFS_NAME).unwrap();
				let raw = c_str.into_raw();
				*vfs_name_ptr = raw;
			}
			SQLITE_OK
		}

		// Configure sqlite3_file methods
		libsqlite3_sys::SQLITE_FCNTL_JOURNAL_POINTER => {
			// Allows configuration of journal file. We can ignore this.
			tracing::debug!("Journal pointer control op");
			SQLITE_OK
		}

		// Migration related control ops
		libsqlite3_sys::SQLITE_FCNTL_WIN32_AV_RETRY
		| libsqlite3_sys::SQLITE_FCNTL_PERSIST_WAL
		| libsqlite3_sys::SQLITE_FCNTL_POWERSAFE_OVERWRITE => {
			// We can ignore these ops
			tracing::debug!("Ignoring migration related control op: {}", op);
			SQLITE_OK
		}

		// Handle PRAGMA statements
		libsqlite3_sys::SQLITE_FCNTL_PRAGMA => {
			// Properly handle PRAGMA statements per SQLite docs
			// The arg parameter is an array of pointers to strings (char**)
			if !arg.is_null() {
				let args = arg as *mut *mut i8;
				// Extract pragma name
				let pragma_name = if !(*args.offset(1)).is_null() {
					let c_str = std::ffi::CStr::from_ptr(*args.offset(1));
					c_str.to_string_lossy().to_string()
				} else {
					String::new()
				};

				// Extract pragma argument (if any)
				let pragma_arg = if !(*args.offset(2)).is_null() {
					let c_str = std::ffi::CStr::from_ptr(*args.offset(2));
					Some(c_str.to_string_lossy().to_string())
				} else {
					None
				};

				tracing::debug!("PRAGMA '{}' with arg: {:?}", pragma_name, pragma_arg);

				// Handle specific pragmas here if needed
				// Currently we don't handle any special pragmas, so return SQLITE_NOTFOUND
				// to let SQLite handle them normally

				tracing::debug!(
					"Not handling PRAGMA '{}', letting SQLite handle it",
					pragma_name
				);
				libsqlite3_sys::SQLITE_NOTFOUND
			} else {
				tracing::error!("PRAGMA with null argument array");
				libsqlite3_sys::SQLITE_NOTFOUND
			}
		}

		// Database corruption related ops
		libsqlite3_sys::SQLITE_FCNTL_BUSYHANDLER
		| libsqlite3_sys::SQLITE_FCNTL_TEMPFILENAME
		| libsqlite3_sys::SQLITE_FCNTL_HAS_MOVED
		| libsqlite3_sys::SQLITE_FCNTL_SYNC
		| libsqlite3_sys::SQLITE_FCNTL_FILE_POINTER => {
			// We can ignore these ops
			tracing::debug!("Ignoring database maintenance control op: {}", op);
			SQLITE_OK
		}

		// RBU extensions
		libsqlite3_sys::SQLITE_FCNTL_RBU => {
			tracing::debug!("Ignoring RBU extension control op: {}", op);
			SQLITE_OK
		}

		// Miscellaneous ops
		libsqlite3_sys::SQLITE_FCNTL_MMAP_SIZE => {
			// Ignored for our VFS as we don't use mmap
			tracing::debug!("MMAP size control op ignored");
			SQLITE_OK
		}

		// Optimization ops
		libsqlite3_sys::SQLITE_FCNTL_TRACE => {
			tracing::debug!("Trace control op ignored");
			SQLITE_OK
		}

		libsqlite3_sys::SQLITE_FCNTL_OVERWRITE => {
			// Used to indicate that file might be overwritten with zeros
			tracing::debug!("Overwrite control op - acknowledging");
			SQLITE_OK
		}

		// Corruption recovery
		libsqlite3_sys::SQLITE_FCNTL_CKPT_DONE | libsqlite3_sys::SQLITE_FCNTL_CKPT_START => {
			tracing::debug!("Checkpoint notification op - acknowledging");
			SQLITE_OK
		}

		// External application control ops
		libsqlite3_sys::SQLITE_FCNTL_EXTERNAL_READER => {
			tracing::debug!("External reader control op - acknowledging");
			SQLITE_OK
		}

		libsqlite3_sys::SQLITE_FCNTL_CKSM_FILE => {
			tracing::debug!("Checksum file control op - acknowledging");
			SQLITE_OK
		}

		// Data encryption and security
		libsqlite3_sys::SQLITE_FCNTL_RESET_CACHE => {
			tracing::debug!("Reset cache control op - acknowledging");
			SQLITE_OK
		}

		// Unsupported or unknown ops
		_ => {
            // TODO: Make these a hard error
			//// Log unknown operations as errors for debugging
			//tracing::error!(
			//	"Unknown file control op: {}. This may cause unexpected behavior.",
			//	op
			//);
			//// Return an error to ensure proper handling
			//metrics::record_vfs_error(&format!("unknown_file_control_op_{}", op));
			//SQLITE_IOERR

			tracing::warn!(
				"Unknown file control op: {}. This may cause unexpected behavior.",
				op
			);
            SQLITE_OK
		}
	}
}

pub unsafe extern "C" fn fdb_file_sector_size(_: *mut sqlite3_file) -> c_int {
	// Return a reasonable sector size
	// This doesn't directly apply to FoundationDB, so we can return a sensible default
	tracing::debug!("fdb_file_sector_size called, returning 4096");
	4096
}

pub unsafe extern "C" fn fdb_file_device_characteristics(_: *mut sqlite3_file) -> c_int {
	// Return the device characteristics
	// These flags inform SQLite about the underlying storage properties
	tracing::debug!("fdb_file_device_characteristics called");

	// We return SQLITE_IOCAP_ATOMIC and SQLITE_IOCAP_SAFE_APPEND
	// because FoundationDB transactions are atomic and we ensure complete writes
	SQLITE_IOCAP_ATOMIC | SQLITE_IOCAP_SAFE_APPEND
}

pub unsafe extern "C" fn fdb_file_close(file: *mut sqlite3_file) -> c_int {
	tracing::debug!("fdb_file_close called");

	let fdb_file = match get_file_ptr(file as *mut FdbFile, "fdb_file_close") {
		Ok(f) => f,
		Err(e) => return e,
	};

	let ext = fdb_file.ext.assume_init_mut();

	// Mark the file as closed
	ext.is_open = false;

	SQLITE_OK
}

/*
 * SHM Implementation Functions
 */

/// Map a shared memory region
pub unsafe extern "C" fn fdb_shm_map(
	file: *mut sqlite3_file,
	region_index: c_int,      // Which region to map (0..=9)
	region_size: c_int,       // Size of each region in bytes
	is_write: c_int,          // True if writing to the map is allowed
	pp_out: *mut *mut c_void, // OUT: Returns a pointer to the mapped memory
) -> c_int {
	tracing::debug!(
		"fdb_shm_map called: region_index={}, region_size={}, is_write={}",
		region_index,
		region_size,
		is_write
	);

	let fdb_file = match get_file_ptr(file as *mut FdbFile, "fdb_shm_map") {
		Ok(f) => f,
		Err(e) => return e,
	};

	let ext = fdb_file.ext.assume_init_mut();

	// Initialize the SHM buffer if not already done
	if ext.shm_buffer.is_none() {
		// Calculate total buffer size based on maximum possible regions
		let total_size = region_size as usize * MAX_SHM_REGIONS;
		ext.shm_buffer = Some(vec![0u8; total_size]);
		ext.shm_region_size = region_size as usize;
	}

	// Ensure the requested region is valid
	if region_index < 0 || region_index >= MAX_SHM_REGIONS as c_int {
		tracing::error!("Invalid SHM region index: {}", region_index);
		metrics::record_vfs_error("invalid_shm_region");
		return SQLITE_IOERR;
	}

	// Verify the region size is consistent with our allocated buffer
	if region_size as usize != ext.shm_region_size {
		tracing::error!(
			"SHM region size mismatch: requested={}, allocated={}",
			region_size,
			ext.shm_region_size
		);
		metrics::record_vfs_error("shm_region_size_mismatch");
		return SQLITE_IOERR;
	}

	// Calculate the pointer to the start of the requested region
	if let Some(buffer) = ext.shm_buffer.as_mut() {
		let region_offset = region_index as usize * ext.shm_region_size;
		let region_ptr = buffer.as_mut_ptr().add(region_offset);

		// Return the pointer to the mapped region
		*pp_out = region_ptr as *mut c_void;

		// Increment the map count
		ext.shm_map_count += 1;

		SQLITE_OK
	} else {
		tracing::error!("SHM buffer not initialized");
		metrics::record_vfs_error("shm_buffer_not_initialized");
		SQLITE_IOERR
	}
}

/// Lock a shared memory region
pub unsafe extern "C" fn fdb_shm_lock(
	_file: *mut sqlite3_file,
	_offset: c_int, // Offset into the lock array
	_n: c_int,      // Number of slots to lock
	_flags: c_int,  // Flags (SQLITE_SHM_LOCK, SQLITE_SHM_SHARED, etc.)
) -> c_int {
	// For single reader/writer scenario, all locks always succeed
	tracing::debug!("fdb_shm_lock called (always succeeds for single reader/writer)");
	SQLITE_OK
}

/// Memory barrier for shared memory operations
pub unsafe extern "C" fn fdb_shm_barrier(_file: *mut sqlite3_file) {
	// Memory barrier to ensure writes are visible to other processes
	// For single reader/writer, this is essentially a no-op
	tracing::debug!("fdb_shm_barrier called (no-op for single reader/writer)");

	// Issue a compiler memory fence in case of optimizations
	std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
}

/// Unmap a shared memory region
pub unsafe extern "C" fn fdb_shm_unmap(
	file: *mut sqlite3_file,
	delete_flag: c_int, // If true, delete the mapping
) -> c_int {
	tracing::debug!("fdb_shm_unmap called: delete_flag={}", delete_flag);

	let fdb_file = match get_file_ptr(file as *mut FdbFile, "fdb_shm_unmap") {
		Ok(f) => f,
		Err(e) => return e,
	};

	let ext = fdb_file.ext.assume_init_mut();

	// If we don't have a SHM buffer, nothing to do
	if ext.shm_buffer.is_none() {
		return SQLITE_OK;
	}

	// Decrement the map count or delete if requested
	if delete_flag != 0 || ext.shm_map_count <= 1 {
		// Free the SHM buffer
		ext.shm_buffer = None;
		ext.shm_map_count = 0;
	} else {
		// Just decrement the reference count
		ext.shm_map_count -= 1;
	}

	SQLITE_OK
}

