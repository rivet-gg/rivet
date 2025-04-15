use foundationdb::Database;
use libsqlite3_sys::*;
use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_void};
use std::ptr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing;

use super::super::fdb::metadata::FdbFileMetadata;
use super::general::FdbVfs;
// Import from fdb is now handled through the WalManager
use crate::metrics;
use crate::utils::{LockState, SQLITE_IOERR, SQLITE_OK};
use crate::wal::WalManager;

// Import submodules
pub mod db;
pub mod shm;
pub mod wal;

// Re-export SHM constants for backward compatibility
pub use shm::{SHM_INITIAL_SIZE, SHM_REGION_SIZE};

/// Enum representing the type of SQLite file
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SqliteFileType {
	/// Main database file (.db)
	Database,
	/// Write-Ahead Log file (.db-wal)
	WAL,
	/// Journal file (.db-journal) - needed temporarily during WAL mode initialization
	/// Even when using WAL mode, SQLite briefly creates a journal file during setup
	Journal,
}

impl SqliteFileType {
	/// Maps a file path to its corresponding SqliteFileType
	/// Returns a Result with either the file type or an error if the file extension is not recognized
	pub fn from_path(path: &str) -> Result<Self, &'static str> {
		match path {
			p if p.ends_with(".db-wal") => Ok(SqliteFileType::WAL),
			p if p.ends_with(".db-journal") => Ok(SqliteFileType::Journal),
			p if p.ends_with(".db") => Ok(SqliteFileType::Database),
			_ => Err("Invalid SQLite file extension. Must use .db, .db-wal, or .db-journal"),
		}
	}
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
	/// WAL manager for WAL operations
	pub wal_manager: WalManager,
	/// Whether the journal file exists (only used for journal files)
	pub journal_exists: bool,
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
	xShmMap: Some(shm::fdb_shm_map),
	xShmLock: Some(shm::fdb_shm_lock),
	xShmBarrier: Some(shm::fdb_shm_barrier),
	xShmUnmap: Some(shm::fdb_shm_unmap),
	xFetch: None,
	xUnfetch: None,
};

/// Get file pointer safely
pub unsafe fn get_file_ptr<'a>(file: *mut FdbFile, context: &str) -> Result<&'a mut FdbFile, c_int> {
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
pub unsafe fn get_file_extension<'a>(
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
pub unsafe fn get_file_extension_mut<'a>(
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
pub unsafe fn get_vfs_from_ext<'a>(ext: &'a FdbFileExt, context: &str) -> Result<&'a FdbVfs, c_int> {
	match ext.vfs.as_ref() {
		Some(vfs) => Ok(vfs),
		None => {
			tracing::error!("Null VFS pointer in {}", context);
			metrics::record_vfs_error(&format!("null_vfs_pointer_{}", context));
			Err(SQLITE_IOERR)
		}
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
			wal::read_wal_file(file, file_path, buf, count, offset, ext, vfs)
		}
		SqliteFileType::Journal => {
			// Journal files are mocked and SQLite should never read from them
			// In WAL mode, journal files are only created temporarily during initialization
			tracing::error!(
				"UNEXPECTED journal file read attempt: {}, offset={}, count={}. This indicates a problem in WAL mode setup.", 
				file_path, offset, count
			);
			
			// Always return an error for journal reads - SQLite should not be reading from journal files
			// when we're forcing WAL mode
			metrics::record_vfs_error("journal_read_attempt");
			return SQLITE_IOERR;
		}
		SqliteFileType::Database => {
			db::read_database_file(file, file_path, buf, count, offset, ext, vfs)
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
	let buf_data = std::slice::from_raw_parts(buf as *const u8, count as usize).to_vec();

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
		SqliteFileType::WAL => {
			// Get a reference to the file extension to use the WAL manager
			let ext = fdb_file.ext.assume_init_ref();
			wal::write_wal_file(
				&file_path,
				file_id,
				offset,
				&buf_data,
				page_size as usize,
				ext,
			)
		},
		SqliteFileType::Journal => {
			// For journal files, we provide a minimal implementation that just logs operations
			// In WAL mode, SQLite only uses the journal file temporarily during initialization
			// The journal is created, written to, and then deleted during WAL setup process
			// After initialization, all transaction logging uses the WAL file exclusively
			tracing::info!("Journal write operation: file={}, offset={}, size={}", 
				file_path, offset, buf_data.len());
			
			// Log a preview of the data for debugging purposes
			if !buf_data.is_empty() {
				let preview_size = std::cmp::min(32, buf_data.len());
				tracing::info!("Journal write data preview (first {} bytes): {:?}", 
					preview_size, &buf_data[..preview_size]);
			}
			
			// Mark the journal file as existing now
			let ext_mut = fdb_file.ext.assume_init_mut();
			ext_mut.journal_exists = true;
			tracing::info!("Journal file marked as existing: {}", file_path);
			
			// Return success - the operation is tracked in memory but not persisted
			// This is sufficient for initialization since the journal is temporary
			SQLITE_OK
		},
		SqliteFileType::Database => {
			// For database files
			db::write_regular_file(
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

pub unsafe extern "C" fn fdb_file_truncate(file: *mut sqlite3_file, size: i64) -> c_int {
	tracing::debug!("fdb_file_truncate called: size={}", size);

	let fdb_file = match get_file_ptr(file as *mut FdbFile, "fdb_file_truncate") {
		Ok(f) => f,
		Err(e) => return e,
	};

	let ext = fdb_file.ext.assume_init_ref();
	let file_type = ext.file_type;
	let file_path = ext.path.clone();
	
	tracing::info!("Truncating file: {}, file_type: {:?}, size: {}", file_path, file_type, size);
	
	// Handle truncate based on file type
	match file_type {
		SqliteFileType::WAL => {
			// For WAL files, use the WAL manager
			tracing::info!("WAL file truncate: {}", file_path);
			
			// If new size is same as current, nothing to do
			if ext.metadata.size == size {
				return SQLITE_OK;
			}
			
			// Use the WAL manager to record truncate operation
			let wal_manager = &ext.wal_manager;
			match wal_manager.truncate_wal(&file_path, size) {
				Ok(_) => {
					// Update the in-memory metadata size
					let ext_mut = fdb_file.ext.assume_init_mut();
					ext_mut.metadata.size = size;
					SQLITE_OK
				},
				Err(e) => {
					tracing::error!("Error truncating WAL file: {}", e);
					metrics::record_vfs_error("truncate_wal_transaction_error");
					SQLITE_IOERR
				}
			}
		},
		SqliteFileType::Journal => {
			// For journal files, we don't need to do anything since they're mocked
			tracing::info!("Journal file truncate operation (mocked): {}, size={}", file_path, size);
			
			// Just update the in-memory size
			let ext_mut = fdb_file.ext.assume_init_mut();
			ext_mut.metadata.size = size;
			SQLITE_OK
		},
		SqliteFileType::Database => {
			// For database files, use FDB to update metadata
			let vfs = match get_vfs_from_ext(ext, "fdb_file_truncate") {
				Ok(vfs) => vfs,
				Err(code) => return code,
			};
			
			// Get existing file metadata
			let mut metadata = ext.metadata.clone();
			
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
					tracing::error!("Error truncating database file: {}", e);
					metrics::record_vfs_error("truncate_database_transaction_error");
					SQLITE_IOERR
				}
			}
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

	let file_type = ext.file_type;
	let file_path = &ext.path;
	
	tracing::info!("Getting file size for: {}, file_type: {:?}", file_path, file_type);
	
	// Handle file size operation based on file type
	match file_type {
		SqliteFileType::WAL => {
			// For WAL files, we can use the WAL manager to get the accurate size
			// but for simplicity we'll use the in-memory metadata size
			tracing::info!("WAL file size: {} = {}", file_path, ext.metadata.size);
			*size_out = ext.metadata.size;
		},
		SqliteFileType::Journal => {
			// For journal files (which are mocked), return the tracked in-memory size
			tracing::info!("Journal file size (mocked): {} = {}", file_path, ext.metadata.size);
			*size_out = ext.metadata.size;
		},
		SqliteFileType::Database => {
			// For database files, use the metadata size from FDB
			tracing::info!("Database file size: {} = {}", file_path, ext.metadata.size);
			*size_out = ext.metadata.size;
		}
	}
	
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
	_file: *mut sqlite3_file,
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

		libsqlite3_sys::SQLITE_FCNTL_PDB => {
			// This is used for persistent database binding
			// For our implementation, we can just acknowledge without changing behavior
			tracing::debug!("PDB control op acknowledged");
			SQLITE_OK
		}
		
		libsqlite3_sys::SQLITE_FCNTL_LOCK_TIMEOUT => {
			// Set lock timeout in milliseconds
			tracing::debug!("Lock timeout control op");
			
			if !arg.is_null() {
				let timeout_ptr = arg as *mut c_int;
				let timeout_value = *timeout_ptr;
				
				// Store the current timeout value (for our implementation we don't track this,
				// so we're just echoing back the given value)
				*timeout_ptr = timeout_value;
				
				tracing::debug!("Lock timeout set to {} ms", timeout_value);
			}
			
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
	let file_type = ext.file_type;
	let file_path = ext.path.clone();
	
	tracing::info!("Closing file: {}, file_type: {:?}", file_path, file_type);
	
	// Handle close based on file type
	match file_type {
		SqliteFileType::WAL => {
			// For WAL files, we might need to flush any pending WAL operations
			// with the WAL manager before closing
			tracing::info!("Closing WAL file: {}", file_path);
			
			// Any WAL-specific cleanup would go here
			// For now, just mark as closed
			ext.is_open = false;
		},
		SqliteFileType::Journal => {
			// For journal files, just log that we're closing the mocked file
			tracing::info!("Closing journal file (mocked): {}", file_path);
			ext.is_open = false;
		},
		SqliteFileType::Database => {
			// For database files, just mark as closed
			tracing::info!("Closing database file: {}", file_path);
			ext.is_open = false;
		}
	}
	
	SQLITE_OK
}

// SHM implementation is in the shm.rs module
