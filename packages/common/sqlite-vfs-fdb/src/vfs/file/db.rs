use foundationdb::Database;
use libsqlite3_sys::*;
use std::cmp::min;
use std::os::raw::{c_int, c_void};
use std::slice;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing;
use uuid::Uuid;

use crate::fdb::metadata::FdbFileMetadata;
use crate::metrics;
use crate::utils::{run_fdb_tx, FdbVfsError, SQLITE_IOERR, SQLITE_OK};
use crate::vfs::general::FdbVfs;
use libsqlite3_sys::SQLITE_IOERR_SHORT_READ;

/// Read pages from FoundationDB
pub fn read_pages_from_fdb(
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

// Helper function to write regular file (database or SHM)
pub fn write_regular_file(
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

pub unsafe fn read_database_file(
	_file: *mut sqlite3_file,
	file_path: &str,
	buf: *mut c_void,
	count: c_int,
	offset: i64,
	extension: &crate::vfs::file::FdbFileExt,
	vfs: &crate::vfs::general::FdbVfs,
) -> c_int {
	tracing::info!("Reading from Database file: {}", file_path);
	// Extract data needed for reading
	let file_id = extension.metadata.file_id;
	let page_size = extension.metadata.page_size as i64;
	let file_size = extension.metadata.size;

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
