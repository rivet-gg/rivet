use foundationdb::{api::NetworkAutoStop, Database};
use libsqlite3_sys::*;
use std::ffi::CStr;
use std::sync::Arc;

fn main() {
    // Initialize FoundationDB
    println!("Initializing FoundationDB...");
    let network: NetworkAutoStop = unsafe { foundationdb::boot() };
    let database = Arc::new(
        foundationdb::Database::default().expect("Failed to connect to FoundationDB"),
    );

    // Register all VFS variants
    println!("Registering all VFS variants...");
    sqlite_vfs_fdb::impls::pages::vfs::register_all_vfs_variants(database.clone())
        .expect("Failed to register VFS variants");

    // List all registered VFSs
    println!("Listing all registered VFSs:");
    unsafe {
        let mut current_vfs = sqlite3_vfs_find(std::ptr::null());
        while !current_vfs.is_null() {
            let name = CStr::from_ptr((*current_vfs).zName);
            let name_str = name.to_str().unwrap_or("Invalid UTF-8");
            println!("  - {}", name_str);
            current_vfs = (*current_vfs).pNext;
        }
    }

    println!("VFS registration check complete.");
}