use foundationdb::Database;
use libsqlite3_sys::*;
use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_void};
use std::ptr;
use std::slice;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing;
use bytes::Bytes;

use super::super::fdb::metadata::FdbFileMetadata;
use super::general::FdbVfs;
use crate::metrics;
use crate::impls::pages::utils::{
    run_fdb_tx, Compression, CompressionType, LockState, SQLITE_IOERR, SQLITE_OK, create_compressor
};

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
	/// Compressor implementation for this file
	pub compressor: Box<dyn Compression>,
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
	let compression_type = ext.metadata.compression_type;

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

	// Read pages from FoundationDB
	let read_result = run_fdb_tx(&ext.db, move |tx| {
		// Clone everything that will be moved into the async block
		let file_id = file_id; // This is a UUID which is already Copy
		let _page_size = page_size; // This is a primitive which is Copy
		let file_size = file_size; // This is a primitive which is Copy
		let keyspace = vfs.keyspace.clone(); // Clone for ownership
		let data_buffer_clone = data_buffer.clone(); // Clone for ownership
		let offset = offset; // This is a primitive which is Copy
		let _end_offset = end_offset; // This is a primitive which is Copy
		let start_page = start_page; // This is a primitive which is Copy
		let end_page = end_page; // This is a primitive which is Copy
		let count = count; // This is a primitive which is Copy
		// Capture the compression type
		let compression_type = compression_type; // This is an enum which is Copy
		// Create a new compressor instance
		let compressor = create_compressor(compression_type);

		async move {
			let mut bytes_read = 0;
			let mut pages_read = 0;
			let mut partial_pages = 0;
			let mut data_buffer = data_buffer_clone;

			// Read each page in order
			for page_num in start_page..=end_page {
				let page_key = keyspace.page_key(&file_id, page_num);
				let page_offset = page_num as i64 * page_size;
				let _page_end = std::cmp::min(page_offset + page_size, file_size);

				// Read the page from FoundationDB
				let page_data_result = tx.get(&page_key, false).await?;

				if let Some(page_data_bytes) = page_data_result {
					pages_read += 1;

					// Handle decompression if needed
					let page_data = if compression_type != CompressionType::None {
						// Decompress the page data
						match compressor.decompress(&page_data_bytes, page_size as usize) {
							Ok(decompressed) => decompressed,
							Err(e) => {
								tracing::error!("Error decompressing page data: {:?}", e);
								// If decompression fails, treat as uncompressed
								Bytes::copy_from_slice(&page_data_bytes)
							}
						}
					} else {
						// No compression, use as is
						Bytes::copy_from_slice(&page_data_bytes)
					};

					// Calculate what part of the page we need
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

	// Create variables to store values outside the scope
	let path_for_metrics;
	let file_id;
	let page_size;
	let current_size;
	let keyspace;
	let db;
	let compression_type;
	let write_result;
	
	// Extract all needed data in a separate scope for the immutable borrow
	{
		let ext = fdb_file.ext.assume_init_ref();
		if !ext.is_open {
			metrics::record_vfs_error("file_not_open_write");
			return SQLITE_IOERR;
		}

		// Get file path for metrics
		let file_path = &ext.path;
		path_for_metrics = file_path.to_string(); // Store outside scope

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
		file_id = ext.metadata.file_id;
		page_size = ext.metadata.page_size as i64;
		current_size = ext.metadata.size;
		compression_type = ext.metadata.compression_type;

		// Calculate the range of pages to write
		let start_page = (offset / page_size) as u32;
		let end_offset = offset + (count as i64);
		let end_page = ((end_offset - 1) / page_size) as u32;

		// Get references to VFS data
		keyspace = vfs.keyspace.clone();
		db = vfs.db.clone();
		
		// Create a file path clone for the closure
		let file_path_clone = path_for_metrics.clone();
		
		// Copy the buffer data for closure
		let buf_data_clone = buf_data.clone();

		// Write data to FoundationDB
		write_result = run_fdb_tx(&db, move |tx| {
			// Clone everything that will be moved into the async block
			let file_id = file_id; // This is a UUID which is Copy
			let _page_size = page_size; // This is a primitive which is Copy
			let _current_size = current_size; // This is a primitive which is Copy
			let file_path_clone = file_path_clone.clone(); // Clone for ownership
			let keyspace_clone = keyspace.clone(); // Clone for ownership
			let buf_data_clone = buf_data_clone.clone(); // Clone for ownership
			let offset = offset; // This is a primitive which is Copy
			let _end_offset = end_offset; // This is a primitive which is Copy
			let start_page = start_page; // This is a primitive which is Copy
			let end_page = end_page; // This is a primitive which is Copy
			let compression_type = compression_type; // This is an enum which is Copy
			// Create a new compressor instance
			let compressor = create_compressor(compression_type);

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
									// If compression is enabled, decompress the data first
									if compression_type != CompressionType::None {
										match compressor.decompress(&data, page_size as usize) {
											Ok(decompressed) => {
												// Use our safe helper to create a properly sized BytesMut
												let mut safe_bytes = crate::impls::pages::utils::create_safe_bytes(page_size as usize);
												// Copy decompressed data
												let copy_len = std::cmp::min(decompressed.len(), safe_bytes.len());
												safe_bytes[..copy_len].copy_from_slice(&decompressed[..copy_len]);
												safe_bytes
											},
											Err(e) => {
												tracing::error!("Error decompressing page data: {:?}", e);
												// Use our safe helper to create a properly sized BytesMut
												let mut safe_bytes = crate::impls::pages::utils::create_safe_bytes(page_size as usize);
												// Fall back to treating as uncompressed
												if !data.is_empty() {
													let copy_len = std::cmp::min(data.len(), safe_bytes.len());
													safe_bytes[..copy_len].copy_from_slice(&data[..copy_len]);
												}
												safe_bytes
											}
										}
									} else {
										// No compression, just copy the data
										let mut safe_bytes = crate::impls::pages::utils::create_safe_bytes(page_size as usize);
										// Copy existing data if any
										if !data.is_empty() {
											let copy_len = std::cmp::min(data.len(), safe_bytes.len());
											safe_bytes[..copy_len].copy_from_slice(&data[..copy_len]);
										}
										safe_bytes
									}
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

							// Compress the page if needed, then store it
							if compression_type != CompressionType::None {
								match compressor.compress(&page_data) {
									Ok(compressed_data) => {
										// Write the compressed page to FoundationDB
										tx.set(&page_key, compressed_data.as_ref());
									},
									Err(e) => {
										tracing::error!("Error compressing page data: {:?}", e);
										// Fall back to storing uncompressed
										tx.set(&page_key, &page_data);
									}
								}
							} else {
								// Write the uncompressed page to FoundationDB
								tx.set(&page_key, &page_data);
							}
						} else {
							// Full page write, just use the buffer data directly
							let page_data = &buf_data_clone[buf_start..buf_end];

							// Record page metrics
							metrics::record_page_write(page_data.len(), false);

							// Compress the page if needed, then store it
							if compression_type != CompressionType::None {
								match compressor.compress(page_data) {
									Ok(compressed_data) => {
										// Write the compressed page to FoundationDB
										tx.set(&page_key, compressed_data.as_ref());
									},
									Err(e) => {
										tracing::error!("Error compressing page data: {:?}", e);
										// Fall back to storing uncompressed
										tx.set(&page_key, page_data);
									}
								}
							} else {
								// Write the uncompressed page to FoundationDB
								tx.set(&page_key, page_data);
							}
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
						compression_type,
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

	} // End of the immutable borrow scope for ext
	
	// We've already stored path_for_metrics earlier
	let count_for_metrics = count as usize;
	
	match write_result {
		Ok((size_to_update, _, _)) => {
			// Update the in-memory file size - now we can get a mutable borrow
			(*fdb_file).ext.assume_init_mut().metadata.size = size_to_update;

			// Update file size metrics
			metrics::update_file_size(&path_for_metrics, size_to_update);

			// Record metrics for the write operation
			metrics::record_write_operation(&path_for_metrics, count_for_metrics, true);

			SQLITE_OK
		}
		Err(e) => {
			tracing::error!("Error writing file: {}", e);
			metrics::record_write_operation(&path_for_metrics, count_for_metrics, false);
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

	// Create variables to store values outside the scope
	let path_for_metrics;
	let truncate_result;
	
	// Create a scope for the immutable borrow
	{
		let ext = fdb_file.ext.assume_init_ref();
		if !ext.is_open {
			metrics::record_vfs_error("file_not_open_truncate");
			return SQLITE_IOERR;
		}

		let file_path = &ext.path;
		let current_size = ext.metadata.size;
		
		// Store path for metrics outside the scope
		path_for_metrics = file_path.to_string();

		if size < 0 {
			tracing::error!("Negative size in fdb_file_truncate: {}", size);
			metrics::record_vfs_error("negative_size_truncate");
			return SQLITE_IOERR;
		}

		// If requested size is larger than current size, do nothing as SQLite
		// expects this to be a no-op in that case
		if size >= current_size {
			metrics::record_file_truncate(file_path, size, true);
			return SQLITE_OK;
		}

		// We need to delete all pages beyond the new size
		let vfs = match ext.vfs.as_ref() {
			Some(v) => v,
			None => {
				tracing::error!("Null VFS pointer in fdb_file_truncate");
				metrics::record_vfs_error("null_vfs_pointer_truncate");
				return SQLITE_IOERR;
			}
		};

		// Extract all needed data before entering the closure
		let file_id = ext.metadata.file_id;
		let page_size = ext.metadata.page_size as i64;
		let file_path_clone = ext.path.clone();
		let compression_type = ext.metadata.compression_type;

		// Calculate the last page to keep
		let last_page_to_keep = (size / page_size) as u32;
		let last_page_in_file = ((current_size - 1) / page_size) as u32;

		// Truncate operation in FoundationDB
		truncate_result = run_fdb_tx(&ext.db, move |tx| {
			// Clone for ownership
			let file_id = file_id; // This is a UUID which is Copy
			let _current_size = current_size; // This is a primitive which is Copy
			let size = size; // This is a primitive which is Copy
			let keyspace = vfs.keyspace.clone(); // Clone for ownership
			let path = file_path_clone.clone(); // Clone for ownership
			let last_page_to_keep = last_page_to_keep; // This is a primitive which is Copy
			let last_page_in_file = last_page_in_file; // This is a primitive which is Copy
			let _page_size = page_size; // This is a primitive which is Copy
			let compression_type = compression_type; // This is an enum which is Copy

		async move {
			// Delete all pages beyond the new size
			for page_num in (last_page_to_keep + 1)..=last_page_in_file {
				let page_key = keyspace.page_key(&file_id, page_num);
				tx.clear(&page_key);
			}

			// Update the file metadata
			let metadata_key = keyspace.metadata_key(&path);
			let metadata_result = tx.get(&metadata_key, false).await?;

			// Update metadata with new size
			if let Some(metadata_bytes) = metadata_result {
				if let Ok(mut metadata) = FdbFileMetadata::from_bytes(&metadata_bytes) {
					metadata.size = size;
					metadata.modified_at = SystemTime::now()
						.duration_since(UNIX_EPOCH)
						.unwrap_or_default()
						.as_secs();
					// Preserve the compression type
					metadata.compression_type = compression_type;

					// Write updated metadata
					let new_metadata_bytes = metadata.to_bytes();
					tx.set(&metadata_key, &new_metadata_bytes);
				}
			}

			Ok(size)
		}
	});

	} // End of the immutable borrow scope for ext
	
	match truncate_result {
		Ok(size_to_update) => {			
			// Update the in-memory file size
			(*fdb_file).ext.assume_init_mut().metadata.size = size_to_update;

			// Record metrics for the truncate operation
			metrics::record_file_truncate(&path_for_metrics, size_to_update, true);

			SQLITE_OK
		}
		Err(e) => {
			tracing::error!("Error truncating file: {}", e);
			metrics::record_file_truncate(&path_for_metrics, size, false);
			metrics::record_vfs_error("truncate_transaction_error");
			SQLITE_IOERR
		}
	}
}

pub unsafe extern "C" fn fdb_file_sync(_file: *mut sqlite3_file, _flags: c_int) -> c_int {
	// For FoundationDB, we assume that all writes are immediately durable
	// once they return success, so sync is a no-op
	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_size(file: *mut sqlite3_file, size_out: *mut i64) -> c_int {
	tracing::debug!("fdb_file_size called");

	if size_out.is_null() {
		tracing::error!("Null size_out pointer in fdb_file_size");
		metrics::record_vfs_error("null_size_out_pointer");
		return SQLITE_IOERR;
	}

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_size");
			metrics::record_vfs_error("null_file_pointer_size");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		metrics::record_vfs_error("file_not_open_size");
		return SQLITE_IOERR;
	}

	// Return the size from the file metadata
	*size_out = ext.metadata.size;

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_lock(
	file: *mut sqlite3_file,
	lock_type: c_int,
) -> c_int {
	tracing::debug!("fdb_file_lock called: lock_type={}", lock_type);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_lock");
			metrics::record_vfs_error("null_file_pointer_lock");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_mut();
	if !ext.is_open {
		metrics::record_vfs_error("file_not_open_lock");
		return SQLITE_IOERR;
	}

	// Get the current lock state
	let current = ext.lock_state;
	let requested = LockState::from(lock_type);

	// Log the lock acquisition attempt
	let lock_type_str = match requested {
		LockState::None => "none",
		LockState::Shared => "shared",
		LockState::Reserved => "reserved",
		LockState::Pending => "pending",
		LockState::Exclusive => "exclusive",
	};
	metrics::record_lock_acquisition(lock_type_str);

	// SQLite lock states follow a specific progression
	// For now, we just track the lock state but don't enforce anything
	// since we're not handling distributed access
	ext.lock_state = requested;

	// If we're upgrading the lock, record it as an escalation
	if requested as u8 > current as u8 {
		metrics::record_lock_escalation();
	}

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_unlock(
	file: *mut sqlite3_file,
	lock_type: c_int,
) -> c_int {
	tracing::debug!("fdb_file_unlock called: lock_type={}", lock_type);

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_unlock");
			metrics::record_vfs_error("null_file_pointer_unlock");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_mut();
	if !ext.is_open {
		metrics::record_vfs_error("file_not_open_unlock");
		return SQLITE_IOERR;
	}

	// Update the lock state
	let requested = LockState::from(lock_type);
	ext.lock_state = requested;

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_check_reserved_lock(
	file: *mut sqlite3_file,
	out: *mut c_int,
) -> c_int {
	tracing::debug!("fdb_file_check_reserved_lock called");

	if out.is_null() {
		tracing::error!("Null out pointer in fdb_file_check_reserved_lock");
		metrics::record_vfs_error("null_out_pointer_check_lock");
		return SQLITE_IOERR;
	}

	let fdb_file = match (file as *mut FdbFile).as_mut() {
		Some(f) => f,
		None => {
			tracing::error!("Null file pointer in fdb_file_check_reserved_lock");
			metrics::record_vfs_error("null_file_pointer_check_lock");
			return SQLITE_IOERR;
		}
	};

	let ext = fdb_file.ext.assume_init_ref();
	if !ext.is_open {
		metrics::record_vfs_error("file_not_open_check_lock");
		return SQLITE_IOERR;
	}

	// For now, assume no reserved lock
	// We'll need to implement proper locking when we support concurrent access
	*out = 0;

	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_control(
	_file: *mut sqlite3_file,
	_op: c_int,
	_arg: *mut c_void,
) -> c_int {
	// We don't support any special file control operations
	SQLITE_OK
}

pub unsafe extern "C" fn fdb_file_sector_size(_file: *mut sqlite3_file) -> c_int {
	// Use a default sector size of 4096 bytes, which is common for SSDs
	// This is a hint to SQLite about optimal I/O size
	4096
}

pub unsafe extern "C" fn fdb_file_device_characteristics(_file: *mut sqlite3_file) -> c_int {
	// Indicate that this is an in-memory like storage
	// We have atomic writes, no need for journal syncing
	// Note: be careful with these flags as they can affect SQLite's behavior
	SQLITE_IOCAP_ATOMIC | SQLITE_IOCAP_SAFE_APPEND | SQLITE_IOCAP_SEQUENTIAL
}