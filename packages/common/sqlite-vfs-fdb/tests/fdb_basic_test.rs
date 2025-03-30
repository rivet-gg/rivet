// Simple FoundationDB connectivity test
mod common;
use crate::common::setup_fdb;

#[test]
fn test_fdb_connectivity() {
	println!("Testing basic FoundationDB connectivity...");

	let db = setup_fdb();

	// Use db.run for a transaction to write data
	futures::executor::block_on(db.run(|tx, _| async move {
		// Set a test key
		tx.set(b"test_key", b"test_value");
		Ok(())
	}))
	.expect("Failed to write to FoundationDB");

	// Read data back
	futures::executor::block_on(db.run(|tx, _| async move {
		tx.get(b"test_key", false).await?;
		Ok(())
	}))
	.expect("Failed to read from FoundationDB");
}