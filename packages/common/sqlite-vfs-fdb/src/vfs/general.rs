use foundationdb::{Database, FdbResult};
use libsqlite3_sys::*;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::{Arc, RwLock};

use super::super::fdb::{keyspace::FdbKeySpace, metadata::FdbFileMetadata};
use super::file::{FdbFile, FdbFileExt, FDB_IO_METHODS};
use crate::metrics;
use crate::utils::{run_fdb_tx, FdbVfsError, LockState, DEFAULT_PAGE_SIZE, FDB_VFS_NAME};
use crate::utils::{
	SQLITE_CANTOPEN, SQLITE_IOERR, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_READONLY,
};

/// Main FoundationDB VFS implementation
pub struct FdbVfs {
	pub db: Arc<Database>,
	pub name: String,
	pub keyspace: FdbKeySpace,
	// Locks map to track file locks (currently not used but kept for future concurrent access)
	#[allow(dead_code)]
	pub locks: Arc<RwLock<HashMap<String, LockState>>>,
}

/// Get a registered VFS by name
pub fn get_registered_vfs() -> Option<*mut sqlite3_vfs> {
	let vfs_name = CString::new(FDB_VFS_NAME).ok()?;
	unsafe {
		let vfs_ptr = sqlite3_vfs_find(vfs_name.as_ptr());
		if vfs_ptr.is_null() {
			None
		} else {
			Some(vfs_ptr)
		}
	}
}

impl FdbVfs {
	/// Create a new FdbVfs with a FoundationDB database
	pub fn with_db(db: Arc<Database>) -> Result<Self, FdbVfsError> {
		// Create a keyspace prefix based on a UUID
		// This allows multiple VFS instances to coexist
		let prefix = uuid::Uuid::new_v4().as_bytes().to_vec();
		tracing::info!("Creating FdbVfs with prefix length: {}", prefix.len());
		tracing::info!("Prefix bytes: {:?}", prefix);

		Ok(Self {
			db,
			name: FDB_VFS_NAME.to_string(),
			keyspace: FdbKeySpace::new(&prefix),
			locks: Arc::new(RwLock::new(HashMap::new())),
		})
	}

	/// Get the name of this VFS
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Register this VFS with SQLite
	#[deprecated(note = "Use register_vfs instead for better memory management")]
	pub fn register(&self) -> Result<(), FdbVfsError> {
		tracing::debug!("Registering FdbVfs with name: {}", self.name());
		// Call our new register_vfs function instead
		register_vfs(self.db.clone())
	}

	// Helper method to store file metadata
	pub fn store_metadata(&self, path: &str, metadata: &FdbFileMetadata) -> FdbResult<()> {
		// Start metrics timer for metadata write
		let timer = metrics::record_metadata_write();

		// Use our blocking helper function
		let metadata_key = self.keyspace.metadata_key(path);
		let metadata_bytes = metadata.to_bytes();

		let result = run_fdb_tx(&self.db, move |tx| {
			let metadata_key_clone = metadata_key.clone();
			let metadata_bytes_clone = metadata_bytes.clone();

			async move {
				tx.set(&metadata_key_clone, &metadata_bytes_clone);
				Ok(())
			}
		});

		// Complete metrics
		metrics::complete_metadata_write(&timer);

		// Update file size metrics
		if result.is_ok() {
			metrics::update_file_size(path, metadata.size);
		}

		result
	}

	// Helper method to retrieve file metadata
	pub fn get_metadata(&self, path: &str) -> FdbResult<Option<FdbFileMetadata>> {
		// Start metrics timer for metadata read
		let timer = metrics::record_metadata_read();

		tracing::info!("Getting metadata for path: {}", path);
		tracing::info!(
			"Keyspace prefix: length={}, empty={}",
			self.keyspace.prefix_len(),
			self.keyspace.prefix_is_empty()
		);

		let metadata_key = self.keyspace.metadata_key(path);
		tracing::info!("Generated metadata_key length: {}", metadata_key.len());

		let path_str = path.to_string();

		let result = run_fdb_tx(&self.db, move |tx| {
			let metadata_key_clone = metadata_key.clone();
			let path_clone = path_str.clone();
			tracing::info!(
				"In transaction: Using metadata_key with length: {}",
				metadata_key_clone.len()
			);

			async move {
				tracing::info!("In async block for path: {}", path_clone);
				let result = tx.get(&metadata_key_clone, false).await?;
				tracing::info!("Got result for path {}: {:?}", path_clone, result.is_some());

				if let Some(bytes_vec) = result {
					let bytes = &bytes_vec;
					match FdbFileMetadata::from_bytes(&bytes) {
						Ok(metadata) => Ok(Some(metadata)),
						Err(e) => {
							tracing::warn!("Failed to parse metadata for {}: {}", path_clone, e);
							metrics::record_vfs_error("metadata_parse");
							Ok(None)
						}
					}
				} else {
					Ok(None)
				}
			}
		});

		// Complete metrics
		metrics::complete_metadata_read(&timer);

		result
	}

	// Helper method to delete file metadata and all pages
	pub fn delete_file(&self, path: &str) -> FdbResult<()> {
		// First, get the metadata to find the file ID
		let metadata_opt = self.get_metadata(path)?;

		// Store the actual operation result
		let result = if let Some(metadata) = metadata_opt {
			// File exists, delete it and all its pages
			let keyspace = self.keyspace.clone();
			let db = self.db.clone();
			let path_copy = path.to_string();
			let metadata_key = self.keyspace.metadata_key(&path_copy);

			run_fdb_tx(&db, move |tx| {
				let metadata = metadata; // This is a struct that should be Copy now
				let keyspace_clone = keyspace.clone(); // Clone for ownership
				let path_copy_clone = path_copy.clone(); // Clone for ownership
				let metadata_key_clone = metadata_key.clone(); // Clone for ownership

				async move {
					// Delete metadata
					tx.clear(&metadata_key_clone);

					// Calculate approximately how many pages the file has
					let page_count = (metadata.size + (metadata.page_size as i64) - 1)
						/ (metadata.page_size as i64);

					// Delete all pages
					for page_num in 0..page_count {
						let page_key = keyspace_clone.page_key(&metadata.file_id, page_num as u32);
						tx.clear(&page_key);
					}

					// Delete lock information
					let lock_key = keyspace_clone.lock_key(&path_copy_clone);
					tx.clear(&lock_key);

					Ok(())
				}
			})
		} else {
			// File doesn't exist, nothing to do
			Ok(())
		};

		// Record metrics after completion using the actual result
		let success = result.is_ok();
		metrics::record_file_delete(path, success);

		// Return the original result
		result
	}

	// Helper method to check if a file exists
	pub fn file_exists(&self, path: &str) -> FdbResult<bool> {
		tracing::info!("Checking if file exists: {}", path);
		let metadata_result = self.get_metadata(path)?;
		tracing::info!(
			"File exists check result for {}: {}",
			path,
			metadata_result.is_some()
		);
		Ok(metadata_result.is_some())
	}
}

// SQLite VFS callback implementations
// These are defined at the module level so they're not recreated each time

unsafe extern "C" fn vfs_open(
	vfs_ptr: *mut sqlite3_vfs,
	path: *const c_char,
	file: *mut sqlite3_file,
	flags: c_int,
	out_flags: *mut c_int,
) -> c_int {
	// Start metrics timer for file open operation
	let timer = metrics::start_file_open();
	let mut success = false;
	let path_str_for_metrics;

	let result = {
		if path.is_null() {
			tracing::info!("FDB VFS: open called with null path");
			path_str_for_metrics = "null_path".to_string();
			SQLITE_CANTOPEN
		} else {
			let path_str = CStr::from_ptr(path).to_str().unwrap_or("Invalid UTF-8");
			path_str_for_metrics = path_str.to_string();
			tracing::debug!("FDB VFS: open called for path: {}", path_str);

			// Get the FDB VFS instance from the pAppData field
			let vfs_instance = (*vfs_ptr).pAppData as *const FdbVfs;
			if vfs_instance.is_null() {
				tracing::error!("VFS instance is null in vfs_open");
				metrics::record_vfs_error("null_vfs_instance");
				SQLITE_CANTOPEN
			} else {
				// Set output flags if provided
				if !out_flags.is_null() {
					*out_flags = flags;
				}

				// Check if file exists
				let file_exists = match (*vfs_instance).file_exists(path_str) {
					Ok(exists) => exists,
					Err(e) => {
						tracing::error!("Error checking if file exists: {}", e);
						metrics::record_vfs_error("file_exists_check");
						return SQLITE_CANTOPEN;
					}
				};

				// Handle read-only flag
				let _is_readonly = (flags & SQLITE_OPEN_READONLY) != 0;

				// Check if we're allowed to create the file
				let can_create = (flags & SQLITE_OPEN_CREATE) != 0;

				if !file_exists && !can_create {
					tracing::debug!("File does not exist and SQLITE_OPEN_CREATE not specified");
					SQLITE_CANTOPEN
				} else {
					// Fetch existing metadata or create new metadata
					let metadata = if file_exists {
						match (*vfs_instance).get_metadata(path_str) {
							Ok(Some(metadata)) => metadata,
							Ok(None) => {
								tracing::error!("File exists but metadata not found");
								metrics::record_vfs_error("missing_metadata");
								return SQLITE_CANTOPEN;
							}
							Err(e) => {
								tracing::error!("Error fetching metadata: {}", e);
								metrics::record_vfs_error("fetch_metadata");
								return SQLITE_CANTOPEN;
							}
						}
					} else {
						// Create new metadata
						let new_metadata = FdbFileMetadata::new(DEFAULT_PAGE_SIZE);

						// Save the metadata in FoundationDB
						match (*vfs_instance).store_metadata(path_str, &new_metadata) {
							Ok(_) => new_metadata,
							Err(e) => {
								tracing::error!("Error creating file metadata: {}", e);
								metrics::record_vfs_error("create_metadata");
								return SQLITE_CANTOPEN;
							}
						}
					};

					// Initialize the FdbFile structure
					let fdb_file = match (file as *mut FdbFile).as_mut() {
						Some(f) => f,
						None => {
							tracing::error!("Null file pointer in vfs_open");
							metrics::record_vfs_error("null_file_pointer");
							return SQLITE_CANTOPEN;
						}
					};

					// Initialize the file structure
					fdb_file.base.pMethods = &FDB_IO_METHODS;

					// Initialize the extension part
					let ext = FdbFileExt {
						vfs: vfs_instance,
						path: path_str.to_string(),
						flags,
						lock_state: LockState::None,
						metadata,
						methods: &FDB_IO_METHODS,
						db: (*vfs_instance).db.clone(),
						is_open: true,
					};
					fdb_file.ext = MaybeUninit::new(ext);

					tracing::debug!("Successfully opened file: {}", path_str);
					success = true;
					SQLITE_OK
				}
			}
		}
	};

	// Record metrics for the file open operation
	metrics::record_file_open_result(&path_str_for_metrics, success);
	metrics::complete_file_open(&timer);

	// Update file metrics for this opened file
	if success {
		// Get file size and update metrics
		let fdb_file = (file as *mut FdbFile).as_mut().unwrap();
		let ext = fdb_file.ext.assume_init_ref();
		metrics::update_file_size(&path_str_for_metrics, ext.metadata.size);
	}

	result
}

unsafe extern "C" fn vfs_delete(
	vfs_ptr: *mut sqlite3_vfs,
	path: *const c_char,
	_sync_dir: c_int,
) -> c_int {
	if path.is_null() {
		return SQLITE_IOERR;
	}

	let path_str = CStr::from_ptr(path).to_str().unwrap_or("Invalid UTF-8");
	tracing::debug!("FDB VFS: delete called for path: {}", path_str);

	// Get the FDB VFS instance from the pAppData field
	let vfs_instance = (*vfs_ptr).pAppData as *const FdbVfs;
	if vfs_instance.is_null() {
		tracing::error!("VFS instance is null in vfs_delete");
		return SQLITE_IOERR;
	}

	// Delete the file
	match (*vfs_instance).delete_file(path_str) {
		Ok(_) => {
			tracing::debug!("Successfully deleted file: {}", path_str);
			SQLITE_OK
		}
		Err(e) => {
			tracing::error!("Error deleting file: {}", e);
			SQLITE_IOERR
		}
	}
}

unsafe extern "C" fn vfs_access(
	vfs_ptr: *mut sqlite3_vfs,
	path: *const c_char,
	flags: c_int,
	res_out: *mut c_int,
) -> c_int {
	if path.is_null() || res_out.is_null() {
		return SQLITE_IOERR;
	}

	let path_str = CStr::from_ptr(path).to_str().unwrap_or("Invalid UTF-8");
	tracing::debug!(
		"FDB VFS: access called for path: {} with flags: {}",
		path_str,
		flags
	);

	// Get the FDB VFS instance from the pAppData field
	let vfs_instance = (*vfs_ptr).pAppData as *const FdbVfs;
	if vfs_instance.is_null() {
		tracing::error!("VFS instance is null in vfs_access");
		return SQLITE_IOERR;
	}

	// Check if the file exists in FoundationDB
	match (*vfs_instance).file_exists(path_str) {
		Ok(exists) => {
			*res_out = if exists { 1 } else { 0 };
			SQLITE_OK
		}
		Err(e) => {
			tracing::error!("Error checking if file exists: {}", e);
			SQLITE_IOERR
		}
	}
}

unsafe extern "C" fn vfs_fullpathname(
	_vfs: *mut sqlite3_vfs,
	path: *const c_char,
	nOut: c_int,
	out: *mut c_char,
) -> c_int {
	if path.is_null() || out.is_null() {
		return SQLITE_IOERR;
	}

	let path_str = CStr::from_ptr(path).to_str().unwrap_or("Invalid UTF-8");
	tracing::info!("FDB VFS: fullpathname called for path: {}", path_str);

	// For our FDB VFS, full pathname is the same as the input pathname
	// Because we're not representing a file system hierarchy
	let out_buf = std::slice::from_raw_parts_mut(out as *mut u8, nOut as usize);
	let copy_len = std::cmp::min(path_str.len(), nOut as usize - 1);
	out_buf[..copy_len].copy_from_slice(&path_str.as_bytes()[..copy_len]);
	out_buf[copy_len] = 0; // Null-terminate

	SQLITE_OK
}

unsafe extern "C" fn vfs_current_time_int64(_vfs: *mut sqlite3_vfs, time_out: *mut i64) -> c_int {
	if !time_out.is_null() {
		// Get the current system time in milliseconds since Unix epoch
		// This is what SQLite expects
		let time = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
			Ok(duration) => duration.as_millis() as i64,
			Err(_) => {
				tracing::error!("Error getting system time");
				return SQLITE_IOERR;
			}
		};

		*time_out = time;
		SQLITE_OK
	} else {
		SQLITE_IOERR
	}
}

unsafe extern "C" fn vfs_current_time(vfs: *mut sqlite3_vfs, time_out: *mut f64) -> c_int {
	if time_out.is_null() {
		return SQLITE_IOERR;
	}

	let mut time_ms: i64 = 0;
	let result = vfs_current_time_int64(vfs, &mut time_ms);
	if result == SQLITE_OK && !time_out.is_null() {
		// Convert from milliseconds to days
		*time_out = time_ms as f64 / 86_400_000.0;
		return SQLITE_OK;
	}
	result
}

pub const VFS_NAME: &str = "fdbpages";

/// Register a FDB VFS with SQLite, creating a stable instance that won't be moved in memory
pub fn register_vfs(db: Arc<Database>) -> Result<(), FdbVfsError> {
	tracing::info!("Creating and registering a SQLite VFS");

	// Create a keyspace prefix based on a UUID
	// This allows multiple VFS instances to coexist
	let prefix = uuid::Uuid::new_v4().as_bytes().to_vec();
	tracing::info!("Creating FdbVfs with prefix length: {}", prefix.len());
	tracing::info!("Prefix bytes: {:?}", prefix);

	// Create the VFS instance in a Box to ensure stable memory location
	let vfs = Box::new(FdbVfs {
		db,
		name: FDB_VFS_NAME.to_string(),
		keyspace: FdbKeySpace::new(&prefix),
		locks: Arc::new(RwLock::new(HashMap::new())),
	});

	// Convert Box to a raw pointer that will be stored in the SQLite VFS structure
	// We intentionally leak this memory since SQLite will use it for the lifetime of the application
	let vfs_raw_ptr = Box::into_raw(vfs);

	unsafe extern "C" fn vfs_open(
		vfs_ptr: *mut sqlite3_vfs,
		path: *const c_char,
		file: *mut sqlite3_file,
		flags: c_int,
		out_flags: *mut c_int,
	) -> c_int {
		if path.is_null() {
			tracing::info!("FDB VFS: open called with null path");
			return SQLITE_CANTOPEN;
		}

		let path_str = CStr::from_ptr(path).to_str().unwrap_or("Invalid UTF-8");
		tracing::debug!("FDB VFS: open called for path: {}", path_str);

		// Get the FDB VFS instance from the pAppData field
		let vfs_instance = (*vfs_ptr).pAppData as *const FdbVfs;
		if vfs_instance.is_null() {
			tracing::error!("VFS instance is null in vfs_open");
			return SQLITE_CANTOPEN;
		}

		// Set output flags if provided
		if !out_flags.is_null() {
			*out_flags = flags;
		}

		// Check if file exists
		let file_exists = match (*vfs_instance).file_exists(path_str) {
			Ok(exists) => exists,
			Err(e) => {
				tracing::error!("Error checking if file exists: {}", e);
				return SQLITE_CANTOPEN;
			}
		};

		// Handle read-only flag
		let _is_readonly = (flags & SQLITE_OPEN_READONLY) != 0;

		// Check if we're allowed to create the file
		let can_create = (flags & SQLITE_OPEN_CREATE) != 0;

		if !file_exists && !can_create {
			tracing::debug!("File does not exist and SQLITE_OPEN_CREATE not specified");
			return SQLITE_CANTOPEN;
		}

		// Fetch existing metadata or create new metadata
		let metadata = if file_exists {
			match (*vfs_instance).get_metadata(path_str) {
				Ok(Some(metadata)) => metadata,
				Ok(None) => {
					tracing::error!("File exists but metadata not found");
					return SQLITE_CANTOPEN;
				}
				Err(e) => {
					tracing::error!("Error fetching metadata: {}", e);
					return SQLITE_CANTOPEN;
				}
			}
		} else {
			// Create new metadata
			let new_metadata = FdbFileMetadata::new(DEFAULT_PAGE_SIZE);

			// Save the metadata in FoundationDB
			match (*vfs_instance).store_metadata(path_str, &new_metadata) {
				Ok(_) => new_metadata,
				Err(e) => {
					tracing::error!("Error creating file metadata: {}", e);
					return SQLITE_CANTOPEN;
				}
			}
		};

		// Initialize the FdbFile structure
		let fdb_file = match (file as *mut FdbFile).as_mut() {
			Some(f) => f,
			None => {
				tracing::error!("Null file pointer in vfs_open");
				return SQLITE_CANTOPEN;
			}
		};

		// Initialize the file structure
		fdb_file.base.pMethods = &FDB_IO_METHODS;

		// Initialize the extension part
		let ext = FdbFileExt {
			vfs: vfs_instance,
			path: path_str.to_string(),
			flags,
			lock_state: LockState::None,
			metadata,
			methods: &FDB_IO_METHODS,
			db: (*vfs_instance).db.clone(),
			is_open: true,
		};
		fdb_file.ext = MaybeUninit::new(ext);

		tracing::debug!("Successfully opened file: {}", path_str);
		SQLITE_OK
	}

	unsafe extern "C" fn vfs_delete(
		vfs_ptr: *mut sqlite3_vfs,
		path: *const c_char,
		_sync_dir: c_int,
	) -> c_int {
		if path.is_null() {
			return SQLITE_IOERR;
		}

		let path_str = CStr::from_ptr(path).to_str().unwrap_or("Invalid UTF-8");
		tracing::debug!("FDB VFS: delete called for path: {}", path_str);

		// Get the FDB VFS instance from the pAppData field
		let vfs_instance = (*vfs_ptr).pAppData as *const FdbVfs;
		if vfs_instance.is_null() {
			tracing::error!("VFS instance is null in vfs_delete");
			return SQLITE_IOERR;
		}

		// Delete the file
		match (*vfs_instance).delete_file(path_str) {
			Ok(_) => {
				tracing::debug!("Successfully deleted file: {}", path_str);
				SQLITE_OK
			}
			Err(e) => {
				tracing::error!("Error deleting file: {}", e);
				SQLITE_IOERR
			}
		}
	}

	unsafe extern "C" fn vfs_access(
		vfs_ptr: *mut sqlite3_vfs,
		path: *const c_char,
		flags: c_int,
		res_out: *mut c_int,
	) -> c_int {
		if path.is_null() || res_out.is_null() {
			return SQLITE_IOERR;
		}

		let path_str = CStr::from_ptr(path).to_str().unwrap_or("Invalid UTF-8");
		tracing::debug!(
			"FDB VFS: access called for path: {} with flags: {}",
			path_str,
			flags
		);

		// Get the FDB VFS instance from the pAppData field
		let vfs_instance = (*vfs_ptr).pAppData as *const FdbVfs;
		if vfs_instance.is_null() {
			tracing::error!("VFS instance is null in vfs_access");
			return SQLITE_IOERR;
		}

		// Check if the file exists in FoundationDB
		match (*vfs_instance).file_exists(path_str) {
			Ok(exists) => {
				*res_out = if exists { 1 } else { 0 };
				SQLITE_OK
			}
			Err(e) => {
				tracing::error!("Error checking if file exists: {}", e);
				SQLITE_IOERR
			}
		}
	}

	unsafe extern "C" fn vfs_fullpathname(
		_vfs: *mut sqlite3_vfs,
		path: *const c_char,
		out_len: c_int,
		out: *mut c_char,
	) -> c_int {
		let path_str = CStr::from_ptr(path).to_str().unwrap_or("Invalid UTF-8");
		tracing::info!("FDB VFS: fullpathname called for path: {}", path_str);

		// Just copy the path as is
		let path_bytes = CStr::from_ptr(path).to_bytes_with_nul();
		if path_bytes.len() <= out_len as usize {
			ptr::copy_nonoverlapping(path, out, path_bytes.len());
			return SQLITE_OK;
		}

		SQLITE_IOERR
	}

	unsafe extern "C" fn vfs_current_time_int64(
		_vfs: *mut sqlite3_vfs,
		time_out: *mut i64,
	) -> c_int {
		if !time_out.is_null() {
			// Get current time in milliseconds since Unix epoch
			if let Ok(time) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
				// Convert to Julian Day Number times 86_400_000 (milliseconds in a day)
				// Unix epoch (1970-01-01) is 2440587.5 in Julian Day
				let julian_day_milliseconds =
					(2440587.5 * 86_400_000.0) as i64 + time.as_millis() as i64;
				*time_out = julian_day_milliseconds;
				return SQLITE_OK;
			}
		}
		SQLITE_IOERR
	}

	unsafe extern "C" fn vfs_current_time(vfs: *mut sqlite3_vfs, time_out: *mut f64) -> c_int {
		let mut time_ms: i64 = 0;
		let result = vfs_current_time_int64(vfs, &mut time_ms);
		if result == SQLITE_OK && !time_out.is_null() {
			// Convert from milliseconds to days
			*time_out = time_ms as f64 / 86_400_000.0;
			return SQLITE_OK;
		}
		result
	}

	// Create a new CString for the VFS name and clone it for later use
	let vfs_name = CString::new(FDB_VFS_NAME).unwrap();
	let vfs_name_for_check = vfs_name.clone();

	// Create a new VFS structure for SQLite
	let vfs_ptr = unsafe {
		// Allocate memory for the VFS structure
		let layout = std::alloc::Layout::new::<sqlite3_vfs>();
		let vfs_ptr = std::alloc::alloc(layout) as *mut sqlite3_vfs;

		if vfs_ptr.is_null() {
			return Err(FdbVfsError::Other(
				"Failed to allocate memory for VFS".to_string(),
			));
		}

		// Initialize the VFS with our implementation functions
		// These are now defined at the module level
		(*vfs_ptr) = sqlite3_vfs {
			iVersion: 1,
			szOsFile: std::mem::size_of::<FdbFile>() as c_int,
			mxPathname: 512,
			pNext: ptr::null_mut(),
			zName: vfs_name.as_ptr(),
			pAppData: ptr::null_mut(),
			xOpen: Some(vfs_open),
			xDelete: Some(vfs_delete),
			xAccess: Some(vfs_access),
			xFullPathname: Some(vfs_fullpathname),
			xDlOpen: None,
			xDlError: None,
			xDlSym: None,
			xDlClose: None,
			xRandomness: None,
			xSleep: None,
			xCurrentTime: Some(vfs_current_time),
			xGetLastError: None,
			xCurrentTimeInt64: Some(vfs_current_time_int64),
			xSetSystemCall: None,
			xGetSystemCall: None,
			xNextSystemCall: None,
		};

		// Set the VFS instance in the pAppData field
		(*vfs_ptr).pAppData = vfs_raw_ptr as *mut c_void;

		// Keep the CString alive to prevent dangling pointer
		Box::leak(Box::new(vfs_name));

		vfs_ptr
	};

	// Register the VFS with SQLite
	let result = unsafe { sqlite3_vfs_register(vfs_ptr, 0) };

	if result != SQLITE_OK {
		return Err(FdbVfsError::Sqlite(result));
	}

	// Verify the VFS was registered
	unsafe {
		let check_vfs = sqlite3_vfs_find(vfs_name_for_check.as_ptr());
		if check_vfs.is_null() {
			tracing::error!("Failed to find registered VFS after registration");
			return Err(FdbVfsError::Other(
				"VFS was registered but not found afterwards".to_string(),
			));
		}

		// List all registered VFSs
		tracing::info!("Checking registered VFSs after registration:");
		let mut current_vfs = sqlite3_vfs_find(ptr::null());
		let mut found = false;

		while !current_vfs.is_null() {
			let name = CStr::from_ptr((*current_vfs).zName);
			let name_str = name.to_str().unwrap_or("Invalid UTF-8");
			tracing::info!("Found VFS: {}", name_str);

			if name_str == FDB_VFS_NAME {
				found = true;
			}

			current_vfs = (*current_vfs).pNext;
		}

		if !found {
			return Err(FdbVfsError::Other(
				"VFS is not in the list of registered VFSs".to_string(),
			));
		}
	}

	tracing::info!("Successfully registered FdbVfs with SQLite");
	Ok(())
}
