mod common;

use crate::common::{FdbSqliteDriver, SqliteDriver, SqliteTestContext, setup_fdb, run_query, run_sql, close_db};
use std::error::Error;

// VFS implementation specific imports
use libsqlite3_sys::sqlite3_vfs;
use sqlite_vfs_fdb::impls::pages::utils::FdbVfsError;
use sqlite_vfs_fdb::impls::pages::vfs::general::FdbVfs;
use sqlite_vfs_fdb::impls::pages::vfs::get_registered_vfs;

// A helper function to set up the FDB VFS and register it, returning the VFS pointer
fn setup_fdb_vfs() -> Result<*mut sqlite3_vfs, FdbVfsError> {
    let db = setup_fdb();
    
    // Create the VFS instance and register it
    let vfs = FdbVfs::with_db(db)?;
    
    // Register the VFS (deprecated method, but we need to keep it for now due to the function signature)
    // In a real implementation, you'd want to store the VFS instance to prevent it from being dropped
    #[allow(deprecated)]
    vfs.register()?;
    
    // Get the registered VFS pointer
    let vfs_ptr = get_registered_vfs().expect("VFS should be registered");
    
    Ok(vfs_ptr)
}

// Test VFS driver registration
#[test]
fn test_pages_vfs_registration() -> Result<(), Box<dyn Error>> {
    let db = setup_fdb();
    let driver = FdbSqliteDriver::new(db);
    
    // Register the driver
    driver.register()?;
    
    // The driver.register() calls impls::pages::vfs::register_vfs
    // Simply verifying that this doesn't throw an error
    
    Ok(())
}

// Run the comprehensive test suite for the 'pages' VFS implementation
sqlite_vfs_tests!(
    // Driver initializer
    || {
        let db = setup_fdb();
        FdbSqliteDriver::new(db)
    },
    // VFS initializer
    setup_fdb_vfs,
    // Test prefix
    "impl_pages"
);