// No need to import here since this is just defining a macro
// The imports will be handled by the file using the macro

// Define a macro for creating a test that will export metrics at the end
#[macro_export]
macro_rules! define_test_with_metrics {
    (
        $name:ident, 
        $body:expr,
        $test_prefix:expr
    ) => {
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            // Run the actual test
            let result = $body();
            
            // Export metrics regardless of test result
            {
                use prometheus::{self, TextEncoder, Encoder};
                use std::fs::File;
                use std::io::Write;
                use std::path::PathBuf;
                
                // Gather and encode metrics
                let mut buffer = Vec::new();
                let encoder = TextEncoder::new();
                let metric_families = prometheus::gather();
                encoder.encode(&metric_families, &mut buffer).unwrap();
                let metrics_output = String::from_utf8(buffer).unwrap();
                
                // Create metrics file in target directory
                let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                file_path.push("target");
                std::fs::create_dir_all(&file_path).unwrap();
                file_path.push(format!("sqlite_vfs_fdb_metrics_{}_{}.txt", $test_prefix, stringify!($name)));
                
                // Write metrics to file
                let mut file = File::create(&file_path).unwrap();
                file.write_all(metrics_output.as_bytes()).unwrap();
                
                // Print the file path
                println!("Metrics for {} exported to: {}", stringify!($name), file_path.display());
            }
            
            // Return the test result
            result
        }
    };
}

// Define a comprehensive test macro that runs all tests for a SQLite VFS implementation
#[macro_export]
macro_rules! sqlite_vfs_tests {
    // Full test suite - both high-level SQLite operations and low-level VFS operations
    ($driver_init:expr, $vfs_init:expr, $test_prefix:expr) => {
        use std::ffi::{c_void, CString};
        use libsqlite3_sys::{sqlite3_file, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_READWRITE};
        use uuid::Uuid;

        // Generate a unique database name for tests
        fn test_db_name(prefix: &str) -> String {
            format!("{}_{}", prefix, Uuid::new_v4())
        }

        // ============= High-level SQLite API Tests =============
        
        define_test_with_metrics!(
            test_create_and_insert,
            || {
                let driver = $driver_init();
                let ctx = SqliteTestContext::new(
                    &format!("{}_{}", $test_prefix, "create_insert"), 
                    driver
                )?;
                
                // Open the database using our VFS
                let sqlite_db = ctx.open_db()?;
    
                // Create a test table
                run_sql(sqlite_db, "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT, value REAL)")?;
    
                // Insert some test data
                run_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Alice', 42.5)")?;
                run_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Bob', 37.0)")?;
                run_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Charlie', 99.9)")?;
    
                // Verify data count
                let count = run_query(sqlite_db, "SELECT COUNT(*) FROM test")?;
                assert_eq!(count, 3, "Should have 3 rows in the table");
    
                // Verify data content
                let sum = run_query(sqlite_db, "SELECT CAST(SUM(value) AS INTEGER) FROM test")?;
                assert_eq!(sum, 179, "Sum of values should be 179");
    
                // Close the database
                close_db(sqlite_db)?;
    
                Ok(())
            }, 
            $test_prefix
        );

        define_test_with_metrics!(
            test_persistence_and_updates,
            || {
                let driver = $driver_init();
                let ctx = SqliteTestContext::new(
                    &format!("{}_{}", $test_prefix, "persistence"), 
                    driver
                )?;
                
                // First create a database with data
                {
                    let sqlite_db = ctx.open_db()?;
                    run_sql(sqlite_db, "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT, value REAL)")?;
                    run_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Alice', 42.5)")?;
                    run_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Bob', 37.0)")?;
                    run_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Charlie', 99.9)")?;
                    close_db(sqlite_db)?;
                }
    
                // Reopen the database and verify persistence
                {
                    let sqlite_db = ctx.open_db()?;
    
                    // Verify the data is still there
                    let count = run_query(sqlite_db, "SELECT COUNT(*) FROM test")?;
                    assert_eq!(count, 3, "Should still have 3 rows after reopening");
    
                    // Update data
                    run_sql(sqlite_db, "UPDATE test SET value = value * 2 WHERE name = 'Alice'")?;
    
                    // Verify update was successful
                    let alice_value = run_query(sqlite_db, "SELECT CAST(value AS INTEGER) FROM test WHERE name = 'Alice'")?;
                    assert_eq!(alice_value, 85, "Alice's value should be updated to 85");
    
                    // Delete a row
                    run_sql(sqlite_db, "DELETE FROM test WHERE name = 'Bob'")?;
    
                    // Verify deletion
                    let count = run_query(sqlite_db, "SELECT COUNT(*) FROM test")?;
                    assert_eq!(count, 2, "Should have 2 rows after deletion");
    
                    close_db(sqlite_db)?;
                }
    
                Ok(())
            },
            $test_prefix
        );

        define_test_with_metrics!(
            test_complex_schema_and_queries,
            || {
                let driver = $driver_init();
                let ctx = SqliteTestContext::new(
                    &format!("{}_{}", $test_prefix, "schema"), 
                    driver
                )?;
                
                // Create a database with complex schema
                let sqlite_db = ctx.open_db()?;
    
                // Create a table with different column types
                run_sql(sqlite_db, "CREATE TABLE complex (
                    id INTEGER PRIMARY KEY,
                    text_data TEXT,
                    int_data INTEGER,
                    real_data REAL,
                    blob_data BLOB,
                    timestamp TEXT
                )")?;
    
                // Insert data with different types
                run_sql(sqlite_db, "INSERT INTO complex (text_data, int_data, real_data, blob_data, timestamp) 
                    VALUES ('Example text', 12345, 123.456, x'DEADBEEF', datetime('now'))")?;
    
                run_sql(sqlite_db, "INSERT INTO complex (text_data, int_data, real_data, blob_data, timestamp) 
                    VALUES ('Another row', 98765, 987.654, x'CAFEBABE', datetime('now', '+1 day'))")?;
    
                // Verify the table has data
                let count = run_query(sqlite_db, "SELECT COUNT(*) FROM complex")?;
                assert_eq!(count, 2, "Should have 2 rows in the complex table");
    
                // Close the database
                close_db(sqlite_db)?;
                
                Ok(())
            },
            $test_prefix
        );

        define_test_with_metrics!(
            test_foreign_keys_and_joins,
            || {
                let driver = $driver_init();
                let ctx = SqliteTestContext::new(
                    &format!("{}_{}", $test_prefix, "foreign_keys"), 
                    driver
                )?;
                
                // Setup database with initial tables
                let sqlite_db = ctx.open_db()?;
                
                // Enable foreign keys
                run_sql(sqlite_db, "PRAGMA foreign_keys = ON")?;
                
                // Create tables with foreign key relationship
                run_sql(sqlite_db, "CREATE TABLE complex (
                    id INTEGER PRIMARY KEY,
                    text_data TEXT,
                    int_data INTEGER
                )")?;
                
                run_sql(sqlite_db, "CREATE TABLE tags (
                    id INTEGER PRIMARY KEY,
                    complex_id INTEGER,
                    tag TEXT,
                    FOREIGN KEY (complex_id) REFERENCES complex(id)
                )")?;
                
                // Insert data
                run_sql(sqlite_db, "INSERT INTO complex (text_data, int_data) VALUES ('First item', 100)")?;
                run_sql(sqlite_db, "INSERT INTO complex (text_data, int_data) VALUES ('Second item', 200)")?;
                
                run_sql(sqlite_db, "INSERT INTO tags (complex_id, tag) VALUES (1, 'important')")?;
                run_sql(sqlite_db, "INSERT INTO tags (complex_id, tag) VALUES (1, 'urgent')")?;
                run_sql(sqlite_db, "INSERT INTO tags (complex_id, tag) VALUES (2, 'normal')")?;
                
                // Query with a join
                let count = run_query(sqlite_db, 
                    "SELECT COUNT(*) FROM complex c JOIN tags t ON c.id = t.complex_id WHERE t.tag = 'important'")?;
                assert_eq!(count, 1, "Should have 1 row with 'important' tag");
                
                // Query total tags
                let count = run_query(sqlite_db, "SELECT COUNT(*) FROM tags")?;
                assert_eq!(count, 3, "Should have 3 tags total");
                
                close_db(sqlite_db)?;
                
                Ok(())
            },
            $test_prefix
        );

        define_test_with_metrics!(
            test_transactions,
            || {
                let driver = $driver_init();
                let ctx = SqliteTestContext::new(
                    &format!("{}_{}", $test_prefix, "transactions"), 
                    driver
                )?;
                
                let sqlite_db = ctx.open_db()?;
                
                // Create test table
                run_sql(sqlite_db, "CREATE TABLE accounts (id INTEGER PRIMARY KEY, name TEXT, balance INTEGER)")?;
                
                // Insert initial data
                run_sql(sqlite_db, "INSERT INTO accounts (name, balance) VALUES ('Account1', 1000)")?;
                run_sql(sqlite_db, "INSERT INTO accounts (name, balance) VALUES ('Account2', 2000)")?;
                
                // Perform a transaction that transfers funds
                run_sql(sqlite_db, "BEGIN TRANSACTION")?;
                run_sql(sqlite_db, "UPDATE accounts SET balance = balance - 500 WHERE name = 'Account1'")?;
                run_sql(sqlite_db, "UPDATE accounts SET balance = balance + 500 WHERE name = 'Account2'")?;
                run_sql(sqlite_db, "COMMIT")?;
                
                // Verify the transaction worked
                let balance1 = run_query(sqlite_db, "SELECT balance FROM accounts WHERE name = 'Account1'")?;
                let balance2 = run_query(sqlite_db, "SELECT balance FROM accounts WHERE name = 'Account2'")?;
                
                assert_eq!(balance1, 500, "Account1 should have 500 remaining");
                assert_eq!(balance2, 2500, "Account2 should have 2500 after transfer");
                
                // Test transaction rollback
                run_sql(sqlite_db, "BEGIN TRANSACTION")?;
                run_sql(sqlite_db, "UPDATE accounts SET balance = balance - 200 WHERE name = 'Account1'")?;
                run_sql(sqlite_db, "ROLLBACK")?;
                
                // Verify the rollback worked
                let balance1_after = run_query(sqlite_db, "SELECT balance FROM accounts WHERE name = 'Account1'")?;
                assert_eq!(balance1_after, 500, "Account1 should still have 500 after rollback");
                
                close_db(sqlite_db)?;
                
                Ok(())
            },
            $test_prefix
        );

        // ============= Low-level VFS Tests =============
        
        define_test_with_metrics!(
            test_vfs_file_create_metadata,
            || -> Result<(), Box<dyn std::error::Error>> {
                // Get the VFS
                let vfs_ptr = $vfs_init()?;
                
                // Test file path
                let test_path = test_db_name(&format!("{}_{}", $test_prefix, "file_metadata_test"));
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
            },
            $test_prefix
        );
        
        define_test_with_metrics!(
            test_vfs_file_read_write_truncate,
            || -> Result<(), Box<dyn std::error::Error>> {
                // Get the VFS
                let vfs_ptr = $vfs_init()?;
                
                // Test file path
                let test_path = test_db_name(&format!("{}_{}", $test_prefix, "read_write_truncate_test"));
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
            },
            $test_prefix
        );
        
        // ============= Concurrent VFS Operations Test =============
        
        define_test_with_metrics!(
            test_concurrent_file_operations,
            || {
                let vfs_ptr = $vfs_init()?;
                
                // This test is a placeholder for a more extensive test of concurrent operations
                // In a real implementation, you would test concurrent reads and writes across multiple threads
                
                assert!(!vfs_ptr.is_null(), "VFS pointer should not be null");
                
                // Just verify the VFS pointer is valid
                unsafe {
                    let version = (*vfs_ptr).iVersion;
                    assert!(version > 0, "VFS version should be greater than 0");
                }
                
                Ok(())
            },
            $test_prefix
        );

        // ============= Simple Basic Operations Tests =============
        
        define_test_with_metrics!(
            test_just_open_database,
            || {
                let driver = $driver_init();
                let ctx = SqliteTestContext::new(
                    &format!("{}_{}", $test_prefix, "just_open"), 
                    driver
                )?;
                
                // Simply open the database using our VFS
                let sqlite_db = ctx.open_db()?;
                
                // Verify the database was opened successfully
                assert!(!sqlite_db.is_null(), "Database should be opened successfully");
                
                // Close the database
                close_db(sqlite_db)?;
                
                Ok(())
            },
            $test_prefix
        );
        
        define_test_with_metrics!(
            test_open_and_insert_one_row,
            || {
                let driver = $driver_init();
                let ctx = SqliteTestContext::new(
                    &format!("{}_{}", $test_prefix, "insert_one"), 
                    driver
                )?;
                
                // Open the database using our VFS
                let sqlite_db = ctx.open_db()?;
                
                // Create a simple table
                run_sql(sqlite_db, "CREATE TABLE simple (id INTEGER PRIMARY KEY, value TEXT)")?;
                
                // Insert a single row
                run_sql(sqlite_db, "INSERT INTO simple (value) VALUES ('test value')")?;
                
                // Verify row count
                let count = run_query(sqlite_db, "SELECT COUNT(*) FROM simple")?;
                assert_eq!(count, 1, "Should have 1 row in the table");
                
                // Close the database
                close_db(sqlite_db)?;
                
                Ok(())
            },
            $test_prefix
        );
        
        define_test_with_metrics!(
            test_open_insert_and_select_one_row,
            || {
                let driver = $driver_init();
                let ctx = SqliteTestContext::new(
                    &format!("{}_{}", $test_prefix, "insert_select_one"), 
                    driver
                )?;
                
                // Open the database using our VFS
                let sqlite_db = ctx.open_db()?;
                
                // Create a simple table
                run_sql(sqlite_db, "CREATE TABLE simple (id INTEGER PRIMARY KEY, value TEXT)")?;
                
                // Insert a single row with a specific value
                run_sql(sqlite_db, "INSERT INTO simple (value) VALUES ('specific test value')")?;
                
                // Verify the specific value was stored correctly
                let value = run_query(sqlite_db, "SELECT length(value) FROM simple WHERE id = 1")?;
                assert_eq!(value, 19, "The value length should be 19 characters");
                
                // Verify we can select the row by its content
                let count = run_query(sqlite_db, "SELECT COUNT(*) FROM simple WHERE value = 'specific test value'")?;
                assert_eq!(count, 1, "Should be able to select the row by its exact value");
                
                // Close the database
                close_db(sqlite_db)?;
                
                Ok(())
            },
            $test_prefix
        );
    };
}