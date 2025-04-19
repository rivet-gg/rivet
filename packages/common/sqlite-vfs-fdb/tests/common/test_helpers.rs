use std::error::Error;
use libsqlite3_sys::sqlite3;
use sqlite_vfs_fdb::{close_sqlite_db, execute_sql, open_sqlite_db, query_count};
use super::setup::test_db_name;

// Define a trait for SQLite driver implementations
pub trait SqliteDriver: Clone {
    fn name(&self) -> &'static str;
    fn register(&self) -> Result<(), Box<dyn Error>>;
}

// FDB implementation of the SQLite driver
#[derive(Clone)]
pub struct FdbSqliteDriver {
    db: std::sync::Arc<foundationdb::Database>,
}

impl FdbSqliteDriver {
    pub fn new(db: std::sync::Arc<foundationdb::Database>) -> Self {
        Self { db }
    }
}

impl SqliteDriver for FdbSqliteDriver {
    fn name(&self) -> &'static str {
        "fdb"
    }

    fn register(&self) -> Result<(), Box<dyn Error>> {
        sqlite_vfs_fdb::impls::pages::vfs::register_vfs(self.db.clone())?;
        Ok(())
    }
}

// Test context that holds state for a specific test
pub struct SqliteTestContext<D: SqliteDriver> {
    pub db_name: String,
    pub driver: D,
}

impl<D: SqliteDriver> SqliteTestContext<D> {
    pub fn new(prefix: &str, driver: D) -> Result<Self, Box<dyn Error>> {
        // Register the driver
        driver.register()?;
        
        // Generate a unique database name for the test
        let db_name = test_db_name(prefix);
        
        Ok(SqliteTestContext { db_name, driver })
    }
    
    pub fn open_db(&self) -> Result<*mut sqlite3, Box<dyn Error>> {
        let db = open_sqlite_db(&self.db_name, self.driver.name())?;
        Ok(db)
    }
}

// Helper functions for working with SQLite databases in tests
pub fn run_query(
    sqlite_db: *mut sqlite3, 
    query: &str
) -> Result<i64, Box<dyn Error>> {
    let count = query_count(sqlite_db, query)?;
    Ok(count)
}

pub fn run_sql(
    sqlite_db: *mut sqlite3, 
    sql: &str
) -> Result<(), Box<dyn Error>> {
    execute_sql(sqlite_db, sql)?;
    Ok(())
}

pub fn close_db(
    sqlite_db: *mut sqlite3
) -> Result<(), Box<dyn Error>> {
    close_sqlite_db(sqlite_db)?;
    Ok(())
}