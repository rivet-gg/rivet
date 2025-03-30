//! SQLite VFS implementation for FoundationDB.
//!
//! This crate provides a virtual file system (VFS) implementation for SQLite
//! that stores data in FoundationDB. This allows SQLite databases to be stored
//! in FoundationDB, providing a distributed, transactional storage backend.
//!
//! # Usage
//!
//! ```ignore
//! use foundationdb::Database;
//! use sqlite_vfs_fdb::{FdbVfs, get_registered_vfs, open_sqlite_db};
//! use std::sync::Arc;
//! use futures::executor::block_on;
//!
//! // Create and initialize the FoundationDB API
//! foundationdb::boot();
//! let network = unsafe { foundationdb::ffi::FDB_CAPI.fdb_setup_network() };
//!
//! // Create a database connection
//! let db = Arc::new(block_on(Database::new(None)).unwrap());
//!
//! // Create and register the VFS
//! let vfs = FdbVfs::with_db(db).unwrap();
//! vfs.register().unwrap();
//!
//! // Use the VFS
//! let db_name = "my_database.db";
//! let sqlite_db = open_sqlite_db(db_name, "fdb").unwrap();
//!
//! // ... use the SQLite database ...
//!
//! // Clean up
//! sqlite_vfs_fdb::close_sqlite_db(sqlite_db).unwrap();
//! ```

mod file;
mod keyspace;
mod metadata;
mod sqlite;
mod utils;
mod vfs;

// Re-export foundationdb for convenience
pub use foundationdb;

// Re-export main types and functions
pub use self::file::{FdbFile, FdbFileExt};
pub use self::keyspace::FdbKeySpace;
pub use self::metadata::FdbFileMetadata;
pub use self::sqlite::{close_sqlite_db, execute_sql, open_sqlite_db, query_count};
pub use self::utils::{FdbVfsError, LockState, DEFAULT_PAGE_SIZE, FDB_VFS_NAME};
pub use self::vfs::{get_registered_vfs, register_vfs, FdbVfs};
