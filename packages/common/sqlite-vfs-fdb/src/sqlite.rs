use libsqlite3_sys::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::utils::{FdbVfsError, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_READWRITE};

/// Open a SQLite database with our FDB VFS
pub fn open_sqlite_db(db_name: &str, vfs_name: &str) -> Result<*mut sqlite3, FdbVfsError> {
	let mut db: *mut sqlite3 = ptr::null_mut();

	tracing::debug!("Attempting to open database with FDB VFS: {}", vfs_name);

	// Create C strings for SQLite API
	let c_db_name = CString::new(db_name).expect("CString conversion failed");

	// Create CString for the VFS name to keep it alive during the function call
	let c_vfs_name = CString::new(vfs_name).expect("CString conversion failed");

	// Open the database with our custom VFS
	let result = unsafe {
		// Use sqlite3_open_v2 to specify our VFS
		sqlite3_open_v2(
			c_db_name.as_ptr(),
			&mut db,
			SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
			c_vfs_name.as_ptr(),
		)
	};

	if result != SQLITE_OK {
		let err_msg = get_sqlite_error(db);
		unsafe { sqlite3_close(db) };
		return Err(FdbVfsError::Other(format!(
			"Failed to open database with VFS '{}': {}",
			vfs_name, err_msg
		)));
	}

	tracing::debug!("Successfully opened database with VFS: {}", vfs_name);
	Ok(db)
}

/// Close a SQLite database
pub fn close_sqlite_db(db: *mut sqlite3) -> Result<(), FdbVfsError> {
	let result = unsafe { sqlite3_close(db) };

	if result != SQLITE_OK {
		let err_msg = get_sqlite_error(db);
		return Err(FdbVfsError::Other(format!(
			"Failed to close database: {}",
			err_msg
		)));
	}

	Ok(())
}

/// Execute a SQL statement on a SQLite database
pub fn execute_sql(db: *mut sqlite3, sql: &str) -> Result<(), FdbVfsError> {
	let c_sql = CString::new(sql).expect("CString conversion failed for SQL statement");

	// Prepare the statement
	let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
	let mut result = unsafe {
		sqlite3_prepare_v2(
			db,
			c_sql.as_ptr(),
			-1, // Read until null terminator
			&mut stmt,
			ptr::null_mut(), // Don't care about remaining SQL
		)
	};

	if result != SQLITE_OK {
		let err_msg = get_sqlite_error(db);
		return Err(FdbVfsError::Other(format!(
			"Failed to prepare statement '{}': {}",
			sql, err_msg
		)));
	}

	// Execute the statement
	result = unsafe { sqlite3_step(stmt) };

	// Check result - could be SQLITE_DONE or SQLITE_ROW for success
	if result != SQLITE_DONE && result != SQLITE_ROW {
		let err_msg = get_sqlite_error(db);
		unsafe { sqlite3_finalize(stmt) };
		return Err(FdbVfsError::Other(format!(
			"Failed to execute statement '{}': {}",
			sql, err_msg
		)));
	}

	// Finalize the statement
	result = unsafe { sqlite3_finalize(stmt) };

	if result != SQLITE_OK {
		let err_msg = get_sqlite_error(db);
		return Err(FdbVfsError::Other(format!(
			"Failed to finalize statement '{}': {}",
			sql, err_msg
		)));
	}

	Ok(())
}

/// Query a count from a SQLite database
pub fn query_count(db: *mut sqlite3, sql: &str) -> Result<i64, FdbVfsError> {
	let c_sql = CString::new(sql).expect("CString conversion failed for SQL query");

	// Prepare the statement
	let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
	let mut result = unsafe {
		sqlite3_prepare_v2(
			db,
			c_sql.as_ptr(),
			-1, // Read until null terminator
			&mut stmt,
			ptr::null_mut(), // Don't care about remaining SQL
		)
	};

	if result != SQLITE_OK {
		let err_msg = get_sqlite_error(db);
		return Err(FdbVfsError::Other(format!(
			"Failed to prepare query '{}': {}",
			sql, err_msg
		)));
	}

	// Execute the statement
	result = unsafe { sqlite3_step(stmt) };

	// Check if we have results
	if result != SQLITE_ROW {
		let err_msg = get_sqlite_error(db);
		unsafe { sqlite3_finalize(stmt) };
		return Err(FdbVfsError::Other(format!(
			"Failed to execute query '{}': {}",
			sql, err_msg
		)));
	}

	// Get the count from the first column
	let count = unsafe { sqlite3_column_int64(stmt, 0) };

	// Finalize the statement
	result = unsafe { sqlite3_finalize(stmt) };

	if result != SQLITE_OK {
		let err_msg = get_sqlite_error(db);
		return Err(FdbVfsError::Other(format!(
			"Failed to finalize query '{}': {}",
			sql, err_msg
		)));
	}

	Ok(count)
}

/// Get the error message from a SQLite database
pub fn get_sqlite_error(db: *mut sqlite3) -> String {
	let error_msg = unsafe { sqlite3_errmsg(db) };

	if error_msg.is_null() {
		return "Unknown error".to_string();
	}

	unsafe {
		CStr::from_ptr(error_msg as *const c_char)
			.to_string_lossy()
			.into_owned()
	}
}
