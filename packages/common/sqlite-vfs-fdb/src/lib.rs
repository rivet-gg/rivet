pub mod fdb;
pub mod metrics;
mod sqlite;
pub mod utils;
pub mod vfs;
pub mod wal;

// Re-export foundationdb for convenience
pub use foundationdb;

// Re-export main functions from sqlite module
pub use sqlite::{close_sqlite_db, execute_sql, open_sqlite_db, query_count};

