use sqlite_vfs_fdb::{close_sqlite_db, execute_sql, open_sqlite_db, query_count, register_vfs};

// Import helper functions from other tests
mod common;
use common::{setup_fdb, test_db_name};

// Shared structure for test state
struct SqliteTestContext {
    db_name: String,
}

// Setup function for shared resources across tests
fn setup_sqlite_test() -> Result<SqliteTestContext, Box<dyn std::error::Error>> {
    // Ensure logging is set up in common.rs

    // Setup FoundationDB
    let db = setup_fdb();

    // Register the VFS with the new stable memory approach
    register_vfs(db)?;

    // Generate a unique database name for the test
    let db_name = test_db_name("sqlite_ops");

    Ok(SqliteTestContext { db_name })
}

#[test]
fn test_create_and_insert() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = setup_sqlite_test()?;
    
    // Open the database using our VFS
    let sqlite_db = open_sqlite_db(&ctx.db_name, "fdb")?;

    // Create a test table
    execute_sql(sqlite_db, "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT, value REAL)")?;

    // Insert some test data
    execute_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Alice', 42.5)")?;
    execute_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Bob', 37.0)")?;
    execute_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Charlie', 99.9)")?;

    // Verify data count
    let count = query_count(sqlite_db, "SELECT COUNT(*) FROM test")?;
    assert_eq!(count, 3, "Should have 3 rows in the table");

    // Verify data content
    let sum = query_count(sqlite_db, "SELECT CAST(SUM(value) AS INTEGER) FROM test")?;
    assert_eq!(sum, 179, "Sum of values should be 179");

    // Close the database
    close_sqlite_db(sqlite_db)?;

    Ok(())
}

#[test]
fn test_persistence_and_updates() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = setup_sqlite_test()?;
    
    // First create a database with data
    {
        let sqlite_db = open_sqlite_db(&ctx.db_name, "fdb")?;
        execute_sql(sqlite_db, "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT, value REAL)")?;
        execute_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Alice', 42.5)")?;
        execute_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Bob', 37.0)")?;
        execute_sql(sqlite_db, "INSERT INTO test (name, value) VALUES ('Charlie', 99.9)")?;
        close_sqlite_db(sqlite_db)?;
    }

    // Reopen the database and verify persistence
    {
        let sqlite_db = open_sqlite_db(&ctx.db_name, "fdb")?;

        // Verify the data is still there
        let count = query_count(sqlite_db, "SELECT COUNT(*) FROM test")?;
        assert_eq!(count, 3, "Should still have 3 rows after reopening");

        // Update data
        execute_sql(sqlite_db, "UPDATE test SET value = value * 2 WHERE name = 'Alice'")?;

        // Verify update was successful
        let alice_value = query_count(sqlite_db, "SELECT CAST(value AS INTEGER) FROM test WHERE name = 'Alice'")?;
        assert_eq!(alice_value, 85, "Alice's value should be updated to 85");

        // Delete a row
        execute_sql(sqlite_db, "DELETE FROM test WHERE name = 'Bob'")?;

        // Verify deletion
        let count = query_count(sqlite_db, "SELECT COUNT(*) FROM test")?;
        assert_eq!(count, 2, "Should have 2 rows after deletion");

        close_sqlite_db(sqlite_db)?;
    }

    Ok(())
}

#[test]
fn test_complex_schema_and_queries() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = setup_sqlite_test()?;
    
    // Create a database with complex schema
    let sqlite_db = open_sqlite_db(&ctx.db_name, "fdb")?;

    // Create a table with different column types
    execute_sql(sqlite_db, "CREATE TABLE complex (
        id INTEGER PRIMARY KEY,
        text_data TEXT,
        int_data INTEGER,
        real_data REAL,
        blob_data BLOB,
        timestamp TEXT
    )")?;

    // Insert data with different types
    execute_sql(sqlite_db, "INSERT INTO complex (text_data, int_data, real_data, blob_data, timestamp) 
        VALUES ('Example text', 12345, 123.456, x'DEADBEEF', datetime('now'))")?;

    execute_sql(sqlite_db, "INSERT INTO complex (text_data, int_data, real_data, blob_data, timestamp) 
        VALUES ('Another row', 98765, 987.654, x'CAFEBABE', datetime('now', '+1 day'))")?;

    // Verify the table has data
    let count = query_count(sqlite_db, "SELECT COUNT(*) FROM complex")?;
    assert_eq!(count, 2, "Should have 2 rows in the complex table");

    // Close the database
    close_sqlite_db(sqlite_db)?;
    
    Ok(())
}

#[test]
fn test_foreign_keys_and_joins() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = setup_sqlite_test()?;
    
    // Setup database with initial tables
    let sqlite_db = open_sqlite_db(&ctx.db_name, "fdb")?;
    
    // Enable foreign keys
    execute_sql(sqlite_db, "PRAGMA foreign_keys = ON")?;
    
    // Create tables with foreign key relationship
    execute_sql(sqlite_db, "CREATE TABLE complex (
        id INTEGER PRIMARY KEY,
        text_data TEXT,
        int_data INTEGER
    )")?;
    
    execute_sql(sqlite_db, "CREATE TABLE tags (
        id INTEGER PRIMARY KEY,
        complex_id INTEGER,
        tag TEXT,
        FOREIGN KEY (complex_id) REFERENCES complex(id)
    )")?;
    
    // Insert data
    execute_sql(sqlite_db, "INSERT INTO complex (text_data, int_data) VALUES ('First item', 100)")?;
    execute_sql(sqlite_db, "INSERT INTO complex (text_data, int_data) VALUES ('Second item', 200)")?;
    
    execute_sql(sqlite_db, "INSERT INTO tags (complex_id, tag) VALUES (1, 'important')")?;
    execute_sql(sqlite_db, "INSERT INTO tags (complex_id, tag) VALUES (1, 'urgent')")?;
    execute_sql(sqlite_db, "INSERT INTO tags (complex_id, tag) VALUES (2, 'normal')")?;
    
    // Query with a join
    let count = query_count(sqlite_db, 
        "SELECT COUNT(*) FROM complex c JOIN tags t ON c.id = t.complex_id WHERE t.tag = 'important'")?;
    assert_eq!(count, 1, "Should have 1 row with 'important' tag");
    
    // Query total tags
    let count = query_count(sqlite_db, "SELECT COUNT(*) FROM tags")?;
    assert_eq!(count, 3, "Should have 3 tags total");
    
    close_sqlite_db(sqlite_db)?;
    
    Ok(())
}

#[test]
fn test_transactions() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = setup_sqlite_test()?;
    
    let sqlite_db = open_sqlite_db(&ctx.db_name, "fdb")?;
    
    // Create test table
    execute_sql(sqlite_db, "CREATE TABLE accounts (id INTEGER PRIMARY KEY, name TEXT, balance INTEGER)")?;
    
    // Insert initial data
    execute_sql(sqlite_db, "INSERT INTO accounts (name, balance) VALUES ('Account1', 1000)")?;
    execute_sql(sqlite_db, "INSERT INTO accounts (name, balance) VALUES ('Account2', 2000)")?;
    
    // Perform a transaction that transfers funds
    execute_sql(sqlite_db, "BEGIN TRANSACTION")?;
    execute_sql(sqlite_db, "UPDATE accounts SET balance = balance - 500 WHERE name = 'Account1'")?;
    execute_sql(sqlite_db, "UPDATE accounts SET balance = balance + 500 WHERE name = 'Account2'")?;
    execute_sql(sqlite_db, "COMMIT")?;
    
    // Verify the transaction worked
    let balance1 = query_count(sqlite_db, "SELECT balance FROM accounts WHERE name = 'Account1'")?;
    let balance2 = query_count(sqlite_db, "SELECT balance FROM accounts WHERE name = 'Account2'")?;
    
    assert_eq!(balance1, 500, "Account1 should have 500 remaining");
    assert_eq!(balance2, 2500, "Account2 should have 2500 after transfer");
    
    // Test transaction rollback
    execute_sql(sqlite_db, "BEGIN TRANSACTION")?;
    execute_sql(sqlite_db, "UPDATE accounts SET balance = balance - 200 WHERE name = 'Account1'")?;
    execute_sql(sqlite_db, "ROLLBACK")?;
    
    // Verify the rollback worked
    let balance1_after = query_count(sqlite_db, "SELECT balance FROM accounts WHERE name = 'Account1'")?;
    assert_eq!(balance1_after, 500, "Account1 should still have 500 after rollback");
    
    close_sqlite_db(sqlite_db)?;
    
    Ok(())
}
