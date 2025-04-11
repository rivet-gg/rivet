#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! rusqlite = { version = "0.34.0", features = ["bundled"] }
//! ```
//!
//! Generates a WAL file for tests to use for testing WAL parsing.

extern crate rusqlite;

use rusqlite::{Connection, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
	// Create fixtures directory
	let fixture_dir = Path::new("tests/fixtures");
	fs::create_dir_all(fixture_dir).expect("Failed to create fixtures directory");

	// Database paths
	let db_path = fixture_dir.join("test_wal.db");
	let wal_path = fixture_dir.join("test_wal.db-wal");
	let shm_path = fixture_dir.join("test_wal.db-shm");
	let backup_wal_path = fixture_dir.join("test_wal");

	// Remove old files if they exist
	for path in [&db_path, &wal_path, &shm_path, &backup_wal_path] {
		if path.exists() {
			fs::remove_file(path).expect("Failed to remove existing file");
		}
	}

	println!("Creating database in WAL mode...");

	// Open a connection to the database
	let conn = Connection::open(&db_path)?;

	// Enable WAL mode
	conn.pragma_update(None, "journal_mode", "WAL")?;

	// Create schema and data
	println!("Creating schema and data...");
	conn.execute(
		"CREATE TABLE test_table (id INTEGER PRIMARY KEY, data TEXT);",
		[],
	)?;
	conn.execute("INSERT INTO test_table VALUES (1, 'first row');", [])?;
	conn.execute("INSERT INTO test_table VALUES (2, 'second row');", [])?;

	// Copy the WAL file to a backup new location while it's still open, since closing database
	// will truncate the WAL
	fs::copy(&wal_path, &backup_wal_path).expect("Failed to backup WAL file");
	println!("Created WAL backup at: {:?}", backup_wal_path);

	// Verify the backup file size
	let backup_size = fs::metadata(&backup_wal_path).unwrap().len();
	assert!(backup_size > 0, "Backup WAL file has zero size!");
	println!("Backup WAL file size: {} bytes", backup_size);

	// Disconnect
	drop(conn);

	// Delete database itself
	for path in [&db_path, &wal_path, &shm_path] {
		if path.exists() {
			fs::remove_file(path).expect("Failed to remove existing file");
		}
	}

	Ok(())
}
