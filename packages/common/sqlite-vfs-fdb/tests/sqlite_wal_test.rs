use rusqlite::{Connection, Result as RusqliteResult};
use sqlite_vfs_fdb::wal::parser::WalParser;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn test_real_sqlite_wal_behavior() -> Result<(), Box<dyn std::error::Error>> {
	// Create a temporary directory for our test
	let temp_dir = tempdir()?;
	let unique_id = Uuid::new_v4();
	let db_path = temp_dir
		.path()
		.join(format!("test_sqlite_{}.db", unique_id));
	let db_path_str = db_path.to_str().unwrap();

	// Helper function to parse WAL and print stats
	let parse_wal = |step: &str| -> Result<(), Box<dyn std::error::Error>> {
		let wal_path = PathBuf::from(format!("{}-wal", db_path_str));

		if !wal_path.exists() {
			println!("Step {}: WAL file does not exist yet", step);
			return Ok(());
		}

		// Read WAL file
		let wal_data = fs::read(&wal_path).unwrap();
		println!("Step {}: WAL file size: {} bytes", step, wal_data.len());

		// If we have at least a header (32 bytes), try to print some info about it
		if wal_data.len() >= 32 {
			let magic_bytes = &wal_data[0..4];
			println!("  WAL magic bytes: {:?}", magic_bytes);

			// Check if magic matches expected value
			let expected_magic = [0x37, 0x7F, 0x06, 0x82];
			if magic_bytes == expected_magic {
				println!("  Magic number is valid (0x377f0682)");
			} else {
				println!("  WARNING: Invalid magic number!");
			}
		}

		// Parse WAL with our parser
		let mut parser = WalParser::new();
		let mut frames = Vec::new();

		parser.add_data(&wal_data);
		match parser.process(|frame| frames.push(frame)) {
			Ok(count) => {
				println!("Step {}: Successfully parsed {} frames", step, count);
				if !frames.is_empty() {
					println!("  First frame page number: {}", frames[0].page_number);
					println!("  First frame db size: {}", frames[0].database_size);
				}
			}
			Err(e) => {
				println!("Step {}: Failed to parse WAL: {}", step, e);
				// Don't fail the test, just log the error
			}
		}

		Ok(())
	};

	// 1. Open SQLite file with WAL mode enabled
	{
		let conn = Connection::open(&db_path)?;
		let _ = conn.query_row("PRAGMA journal_mode=WAL", [], |_| Ok(()))?;
		println!("Opened database in WAL mode: {}", db_path_str);

		// 2. Parse WAL (should be empty or just header)
		parse_wal("2 - Initial WAL")?;

		// 3. Run schema change
		conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)", [])?;
		println!("Created test table");

		// 4. Parse WAL after schema change
		parse_wal("4 - After schema")?;

		// 5. Run simple query (should not modify WAL)
		let count: i64 = conn.query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))?;
		println!("Count result: {}", count);

		// 6. Parse WAL after query
		parse_wal("6 - After query")?;

		// 7. Start transaction and insert
		conn.execute("BEGIN TRANSACTION", [])?;
		conn.execute("INSERT INTO test (id, value) VALUES (1, 'test1')", [])?;
		println!("Started transaction and inserted a row");

		// 8. Parse WAL during transaction
		parse_wal("8 - During transaction")?;

		// 9. Complete transaction
		conn.execute("COMMIT", [])?;
		println!("Committed transaction");

		// 10. Parse WAL after commit
		parse_wal("10 - After commit")?;

		// 11. Close connection explicitly
		drop(conn);
		println!("Closed database connection");
	}

	// 12. Parse WAL after connection close
	parse_wal("12 - After close")?;

	// Optionally: Examine on-disk structure (this helps debug real SQLite behavior)
	if let Ok(metadata) = fs::metadata(format!("{}-wal", db_path_str)) {
		println!("Final WAL file size: {} bytes", metadata.len());
	} else {
		println!("WAL file no longer exists after close (may have been checkpointed)");
	}

	Ok(())
}

