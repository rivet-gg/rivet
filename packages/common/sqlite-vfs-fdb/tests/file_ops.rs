use std::ffi::{c_void, CString};

use libsqlite3_sys::{sqlite3_file, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_READWRITE};
use sqlite_vfs_fdb::{get_registered_vfs, FdbVfs, FdbVfsError};

// Import helper functions from other tests
mod common;
use common::{setup_fdb, test_db_name};

// Comment out tests temporarily - these are low-level tests that need to be updated
// after our refactoring to properly handle the ext structure
#[test]
fn test_file_create_metadata() -> Result<(), FdbVfsError> {
	let db = setup_fdb();

	// Create the VFS instance and register it
	let vfs = FdbVfs::with_db(db)?;
	vfs.register()?;

	// Get the registered VFS pointer
	let vfs_ptr = get_registered_vfs().expect("VFS should be registered");

	// Test file path
	let test_path = test_db_name("file_metadata_test");
	let c_path = CString::new(test_path.clone()).expect("Failed to create CString");

	// Set up for checking if file exists
	let mut res_out: i32 = 0;

	// Check file access initially - should not exist
	unsafe {
		let xaccess = (*vfs_ptr).xAccess.expect("xAccess should be defined");
		let result = xaccess(vfs_ptr, c_path.as_ptr(), 0, &mut res_out);
		assert_eq!(result, SQLITE_OK);
		assert_eq!(res_out, 0, "File shouldn't exist yet");
	}

	// Create a new file
	unsafe {
		// Allocate memory for a file handle
		let file_size = (*vfs_ptr).szOsFile;
		let file_memory = libc::malloc(file_size as usize) as *mut sqlite3_file;
		assert!(!file_memory.is_null(), "Failed to allocate memory for file handle");

		// Zero the memory
		libc::memset(file_memory as *mut libc::c_void, 0, file_size as usize);

		// Open the file
		let mut flags = SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE;
		let xopen = (*vfs_ptr).xOpen.expect("xOpen should be defined");
		let result = xopen(
			vfs_ptr,
			c_path.as_ptr(),
			file_memory,
			flags,
			&mut flags,
		);
		assert_eq!(result, SQLITE_OK, "Failed to create file");

		// Now check if the file exists
		let mut exists_out: i32 = 0;
		let xaccess = (*vfs_ptr).xAccess.expect("xAccess should be defined");
		let result = xaccess(vfs_ptr, c_path.as_ptr(), 0, &mut exists_out);
		assert_eq!(result, SQLITE_OK);
		assert_eq!(exists_out, 1, "File should exist after creation");

		// Write some data to the file
		let test_data = b"Hello, FoundationDB SQLite VFS!";
		let methods = (*file_memory).pMethods;
		assert!(!methods.is_null(), "File methods should not be null");

		let xwrite = (*methods).xWrite.expect("xWrite should be defined");
		let result = xwrite(
			file_memory,
			test_data.as_ptr() as *const c_void,
			test_data.len() as i32,
			0, // offset
		);
		assert_eq!(result, SQLITE_OK, "Failed to write to file");

		// Check the file size
		let mut size: i64 = 0;
		let xfilesize = (*methods).xFileSize.expect("xFileSize should be defined");
		let result = xfilesize(file_memory, &mut size);
		assert_eq!(result, SQLITE_OK, "Failed to get file size");
		assert_eq!(size, test_data.len() as i64, "File size doesn't match written data size");

		// Read the data back
		let read_buffer = libc::malloc(test_data.len()) as *mut u8;
		libc::memset(read_buffer as *mut c_void, 0, test_data.len());

		let xread = (*methods).xRead.expect("xRead should be defined");
		let result = xread(
			file_memory,
			read_buffer as *mut c_void,
			test_data.len() as i32,
			0, // offset
		);
		assert_eq!(result, SQLITE_OK, "Failed to read from file");

		// Compare the data
		let read_data = std::slice::from_raw_parts(read_buffer, test_data.len());
		assert_eq!(read_data, test_data, "Read data doesn't match written data");

		// Free the read buffer
		libc::free(read_buffer as *mut c_void);

		// Close the file
		let xclose = (*methods).xClose.expect("xClose should be defined");
		let result = xclose(file_memory);
		assert_eq!(result, SQLITE_OK, "Failed to close file");

		// Free the file memory
		libc::free(file_memory as *mut c_void);
	}

	// Delete the file
	unsafe {
		let xdelete = (*vfs_ptr).xDelete.expect("xDelete should be defined");
		let result = xdelete(vfs_ptr, c_path.as_ptr(), 0);
		assert_eq!(result, SQLITE_OK, "Failed to delete file");
	}

	// Check file was deleted
	unsafe {
		let xaccess = (*vfs_ptr).xAccess.expect("xAccess should be defined");
		let result = xaccess(vfs_ptr, c_path.as_ptr(), 0, &mut res_out);
		assert_eq!(result, SQLITE_OK);
		assert_eq!(res_out, 0, "File should be deleted");
	}

	Ok(())
}

#[test]
fn test_file_read_write_truncate() -> Result<(), FdbVfsError> {
	let db = setup_fdb();

	// Create the VFS instance and register it
	let vfs = FdbVfs::with_db(db)?;
	vfs.register()?;

	// Get the registered VFS pointer
	let vfs_ptr = get_registered_vfs().expect("VFS should be registered");

	// Test file path
	let test_path = test_db_name("read_write_truncate_test");
	let c_path = CString::new(test_path.clone()).expect("Failed to create CString");

	// Create a new file
	let file_memory = unsafe {
		// Allocate memory for a file handle
		let file_size = (*vfs_ptr).szOsFile;
		let file_memory = libc::malloc(file_size as usize) as *mut sqlite3_file;
		assert!(!file_memory.is_null(), "Failed to allocate memory for file handle");

		// Zero the memory
		libc::memset(file_memory as *mut c_void, 0, file_size as usize);

		// Open the file
		let mut flags = SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE;
		let xopen = (*vfs_ptr).xOpen.expect("xOpen should be defined");
		let result = xopen(
			vfs_ptr,
			c_path.as_ptr(),
			file_memory,
			flags,
			&mut flags,
		);
		assert_eq!(result, SQLITE_OK, "Failed to create file");

		file_memory
	};

	// Write a large amount of data
	unsafe {
		// Test data - 10KB of repeating pattern
		let data_size = 10 * 1024;
		let large_data = (0..data_size).map(|i| (i % 256) as u8).collect::<Vec<u8>>();

		// Write to the file
		let methods = (*file_memory).pMethods;
		let xwrite = (*methods).xWrite.expect("xWrite should be defined");
		let result = xwrite(
			file_memory,
			large_data.as_ptr() as *const c_void,
			data_size as i32,
			0, // offset
		);
		assert_eq!(result, SQLITE_OK, "Failed to write large data to file");

		// Check the file size
		let xfilesize = (*methods).xFileSize.expect("xFileSize should be defined");
		let mut size: i64 = 0;
		let result = xfilesize(file_memory, &mut size);
		assert_eq!(result, SQLITE_OK, "Failed to get file size");
		assert_eq!(size, data_size as i64, "File size doesn't match written data size");

		// Read a portion of the data to verify it
		let read_size = 1024; // Read first 1KB
		let read_buffer = libc::malloc(read_size) as *mut u8;
		assert!(!read_buffer.is_null(), "Failed to allocate read buffer");

		// Zero the buffer
		libc::memset(read_buffer as *mut c_void, 0, read_size);

		// Read from the file
		let xread = (*methods).xRead.expect("xRead should be defined");
		let result = xread(
			file_memory,
			read_buffer as *mut c_void,
			read_size as i32,
			0, // offset
		);
		assert_eq!(result, SQLITE_OK, "Failed to read from file");

		// Verify the first 1KB matches
		let read_data = std::slice::from_raw_parts(read_buffer, read_size);
		for i in 0..read_size {
			assert_eq!(read_data[i], (i % 256) as u8, "Data mismatch at position {}", i);
		}

		// Free the read buffer
		libc::free(read_buffer as *mut c_void);

		// Truncate the file to half its size
		let new_size = data_size as i64 / 2;
		let xtruncate = (*methods).xTruncate.expect("xTruncate should be defined");
		let result = xtruncate(file_memory, new_size);
		assert_eq!(result, SQLITE_OK, "Failed to truncate file");

		// Check the new file size
		let mut size: i64 = 0;
		let result = xfilesize(file_memory, &mut size);
		assert_eq!(result, SQLITE_OK, "Failed to get file size after truncation");
		assert_eq!(size, new_size, "File size doesn't match truncated size");

		// Read from the end of the file (should fail with a short read)
		let read_buffer = libc::malloc(read_size) as *mut u8;
		libc::memset(read_buffer as *mut c_void, 0, read_size);

		let result = xread(
			file_memory,
			read_buffer as *mut c_void,
			read_size as i32,
			new_size, // offset at EOF
		);
		assert_ne!(result, SQLITE_OK, "Reading beyond EOF should return short read");

		// Free the read buffer
		libc::free(read_buffer as *mut c_void);

		// Close the file
		let xclose = (*methods).xClose.expect("xClose should be defined");
		let result = xclose(file_memory);
		assert_eq!(result, SQLITE_OK, "Failed to close file");

		// Free the file memory
		libc::free(file_memory as *mut c_void);
	}

	// Delete the file
	unsafe {
		let xdelete = (*vfs_ptr).xDelete.expect("xDelete should be defined");
		let result = xdelete(vfs_ptr, c_path.as_ptr(), 0);
		assert_eq!(result, SQLITE_OK, "Failed to delete file");
	}

	Ok(())
}
