use libsqlite3_sys::*;
use std::os::raw::{c_int, c_void};
use std::slice;
use std::sync::Arc;
use tracing;
use uuid::Uuid;

use crate::metrics;
use crate::utils::{SQLITE_IOERR, SQLITE_OK};

/// Helper function to write WAL file
pub fn write_wal_file(
	file_path: &str,
	file_id: Uuid,
	offset: i64,
	buf_data: &[u8],
	page_size: usize,
	db: &Arc<foundationdb::Database>,
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

pub unsafe fn read_wal_file(
	_file: *mut sqlite3_file,
	file_path: &str,
	buf: *mut c_void,
	count: c_int,
	offset: i64,
	extension: &crate::vfs::file::FdbFileExt,
	vfs: &crate::vfs::general::FdbVfs,
) -> c_int {
	tracing::info!("Reading from WAL file: {}", file_path);

	// Create WAL manager
	let wal_manager = crate::wal::WalManager::new(extension.db.clone(), vfs.keyspace.clone());

	// Use WAL manager to read data
	match wal_manager.read_wal_data(&extension.metadata.file_id, offset, count as usize) {
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
