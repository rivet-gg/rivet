use std::error::Error;
use libsqlite3_sys::sqlite3;
use sqlite_vfs_fdb::{close_sqlite_db, execute_sql, open_sqlite_db, query_count};
use super::setup::test_db_name;
use std::path::PathBuf;

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
    
    // Convenience method to reduce boilerplate 
    pub fn new_from_db(db: std::sync::Arc<foundationdb::Database>) -> Self {
        Self { db }
    }
}

impl SqliteDriver for FdbSqliteDriver {
    fn name(&self) -> &'static str {
        "fdb"
    }

    fn register(&self) -> Result<(), Box<dyn Error>> {
        sqlite_vfs_fdb::vfs::register_vfs(self.db.clone())?;
        Ok(())
    }
}

// Helper function to export metrics
fn export_metrics(test_name: &str) {
    use prometheus::{self, TextEncoder, Encoder};
    use std::fs::File;
    use std::io::Write;
    
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
    file_path.push(format!("sqlite_vfs_fdb_metrics_fdb_{}.txt", test_name));
    
    // Write metrics to file
    if let Ok(mut file) = File::create(&file_path) {
        if let Err(e) = file.write_all(metrics_output.as_bytes()) {
            eprintln!("Failed to write metrics to file: {}", e);
        } else {
            println!("Metrics for {} exported to: {}", test_name, file_path.display());
        }
    } else {
        eprintln!("Failed to create metrics file");
    }
}

// Test context that holds state for a specific test
pub struct SqliteTestContext<D: SqliteDriver> {
    pub db_name: String,
    pub driver: D,
    // Store the actual test name for metrics
    test_name: String,
}

impl<D: SqliteDriver> SqliteTestContext<D> {
    pub fn new(prefix: &str, driver: D) -> Result<Self, Box<dyn Error>> {
        // Register the driver
        driver.register()?;
        
        // Generate a unique database name for the test
        let db_name = test_db_name(prefix);
        
        // Extract test name from prefix for metrics
        let test_name = if let Some(idx) = prefix.find('_') {
            prefix[idx+1..].to_string()
        } else {
            prefix.to_string()
        };
        
        Ok(SqliteTestContext { db_name, driver, test_name })
    }
    
    pub fn open_db(&self) -> Result<*mut sqlite3, Box<dyn Error>> {
        let db = open_sqlite_db(&self.db_name, self.driver.name())?;
        Ok(db)
    }
}

// Implement Drop trait to automatically export metrics
impl<D: SqliteDriver> Drop for SqliteTestContext<D> {
    fn drop(&mut self) {
        export_metrics(&self.test_name);
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
