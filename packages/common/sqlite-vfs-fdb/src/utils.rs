use foundationdb::{Database, FdbBindingError, FdbError, FdbResult};
use futures::future::Future;
use std::io;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

use crate::metrics;

// Export SQLite constants we need
pub use libsqlite3_sys::SQLITE_CANTOPEN;
pub use libsqlite3_sys::SQLITE_IOERR;
pub use libsqlite3_sys::SQLITE_LOCK_EXCLUSIVE;
pub use libsqlite3_sys::SQLITE_LOCK_NONE;
pub use libsqlite3_sys::SQLITE_LOCK_PENDING;
pub use libsqlite3_sys::SQLITE_LOCK_RESERVED;
pub use libsqlite3_sys::SQLITE_LOCK_SHARED;
pub use libsqlite3_sys::SQLITE_OK;
pub use libsqlite3_sys::SQLITE_OPEN_CREATE;
pub use libsqlite3_sys::SQLITE_OPEN_READONLY;
pub use libsqlite3_sys::SQLITE_OPEN_READWRITE;

// Default page size for SQLite
// IMPORTANT: This must be less than 10,000 since this is the max key size for FoundationDB. If we
// need a larger page size, we can chunk the pages in to multiple keys.
// Limiting the page size to something conservative to avoid capacity issues
pub const DEFAULT_PAGE_SIZE: usize = 4096;
pub const MAX_SAFE_PAGE_SIZE: usize = 8192; // Conservative limit to avoid BytesMut capacity issues

// SQLite VFS module name
pub const FDB_VFS_NAME: &str = "fdb";

// Global counter for transaction retries
static FDB_TRANSACTION_RETRIES: AtomicU64 = AtomicU64::new(0);

/// Helper function to run a FoundationDB transaction in a blocking manner
/// This allows us to use async FoundationDB API in synchronous SQLite VFS methods
pub fn run_fdb_tx<T, F, Fut>(db: &Database, f: F) -> FdbResult<T>
where
	F: Fn(foundationdb::RetryableTransaction) -> Fut + Send + 'static,
	Fut: Future<Output = Result<T, FdbBindingError>> + Send + 'static,
	T: Send + 'static,
{
	// Start metrics timer for the transaction
	let timer = metrics::start_fdb_transaction();

	// Reset retry counter for this transaction
	FDB_TRANSACTION_RETRIES.store(0, Ordering::SeqCst);

	// Here we create a wrapper function that handles the retry flag
	let wrapper = move |tx: foundationdb::RetryableTransaction,
	                    maybe_committed: foundationdb::MaybeCommitted| {
		// If this is a retry, increment the retry counter
		if maybe_committed.into() {
			FDB_TRANSACTION_RETRIES.fetch_add(1, Ordering::SeqCst);
		}
		f(tx)
	};

	// Here we create a future that will be run with the transaction
	let fut = db.run(wrapper);

	// Block on the future and convert FdbBindingError to FdbError
	match futures::executor::block_on(fut) {
		Ok(result) => {
			// Record successful transaction with retries
			let retries = FDB_TRANSACTION_RETRIES.load(Ordering::SeqCst);
			metrics::complete_fdb_transaction(&timer, retries, true);
			Ok(result)
		}
		Err(e) => {
			tracing::error!("Error in FDB transaction: {:?}", e);
			// Record failed transaction
			let retries = FDB_TRANSACTION_RETRIES.load(Ordering::SeqCst);
			metrics::complete_fdb_transaction(&timer, retries, false);

			// Record error in metrics
			metrics::record_vfs_error("fdb_transaction");

			// Convert from FdbBindingError to FdbError
			Err(FdbError::from_code(1))
		}
	}
}

/// Error types for the FoundationDB VFS implementation
#[derive(Error, Debug)]
pub enum FdbVfsError {
	#[error("SQLite error: {0}")]
	Sqlite(i32),

	#[error("FoundationDB error: {0}")]
	Fdb(#[from] FdbError),

	#[error("I/O error: {0}")]
	Io(#[from] io::Error),

	#[error("{0}")]
	Other(String),
}

impl From<&str> for FdbVfsError {
	fn from(msg: &str) -> Self {
		FdbVfsError::Other(msg.to_string())
	}
}

impl From<String> for FdbVfsError {
	fn from(msg: String) -> Self {
		FdbVfsError::Other(msg)
	}
}

/// Lock state for a file
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LockState {
	None,
	Shared,
	Reserved,
	Pending,
	Exclusive,
}

impl From<c_int> for LockState {
	fn from(value: c_int) -> Self {
		match value {
			SQLITE_LOCK_NONE => LockState::None,
			SQLITE_LOCK_SHARED => LockState::Shared,
			SQLITE_LOCK_RESERVED => LockState::Reserved,
			SQLITE_LOCK_PENDING => LockState::Pending,
			SQLITE_LOCK_EXCLUSIVE => LockState::Exclusive,
			_ => LockState::None,
		}
	}
}

/// Safe BytesMut creation to avoid capacity overflows
pub fn create_safe_bytes(size: usize) -> bytes::BytesMut {
	// Check if the size is reasonable and chunk if needed
	let safe_size = std::cmp::min(size, MAX_SAFE_PAGE_SIZE);
	let buffer = vec![0u8; safe_size];
	bytes::BytesMut::from(&buffer[..])
}
