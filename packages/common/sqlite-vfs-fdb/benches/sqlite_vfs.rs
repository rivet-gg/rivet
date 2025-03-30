//! Run with:
//!
//! ```sh
//! cargo bench -q -p sqlite-vfs-fdb --bench sqlite_vfs
//! ````
//!
//! Or for faster iteration:
//!
//! ```sh
//! cargo bench -q -p sqlite-vfs-fdb --bench sqlite_vfs --profile dev
//! ````

use divan::Bencher;
use foundationdb::{api::NetworkAutoStop, Database};
use lazy_static::lazy_static;
use sqlite_vfs_fdb::{close_sqlite_db, execute_sql, open_sqlite_db, query_count, register_vfs};
use std::sync::Arc;
use tracing;
use tracing_subscriber;
use uuid::Uuid;

lazy_static! {
	// Hold onto the network object for the lifetime of the program
	static ref NETWORK: NetworkAutoStop = {
		tracing::info!("Initializing FoundationDB for tests...");
		unsafe { foundationdb::boot() }
	};

	static ref DATABASE: Arc<Database> = {
		// Make sure network is initialized first by referencing it
		let _ = &*NETWORK;

		Arc::new(foundationdb::Database::default().expect("Failed to connect to FoundationDB"))
	};
}

//const ROW_COUNTS: &[usize] = &[1];
const ROW_COUNTS: &[usize] = &[10_000];
const SAMPLE_SIZE: u32 = 1;
const SAMPLE_COUNT: u32 = 10;

fn main() {
	// Initialize logging only if RUST_LOG is set
	let _ = tracing_subscriber::fmt()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init();

	// Register the VFS with the new stable memory approach
	register_vfs(DATABASE.clone()).expect("Failed to register VFS");

	// Start the benchmarks
	divan::main();
}

// Generate a unique database name for benchmarks
fn bench_db_name(prefix: &str) -> String {
	format!("{}_{}", prefix, Uuid::new_v4())
}

// Setup function for the benchmark environment
fn setup_sqlite() -> (Arc<Database>, String) {
	// Setup FoundationDB
	let db = DATABASE.clone();

	// Register the VFS
	register_vfs(db.clone()).expect("Failed to register VFS");

	// Generate a unique database name
	let db_name = bench_db_name("bench");

	(db, db_name)
}

// Helper to create a database with given size
fn setup_database(db_name: &str, row_count: usize) -> *mut libsqlite3_sys::sqlite3 {
	// Open the database
	let sqlite_db = open_sqlite_db(db_name, "fdb").expect("Failed to open SQLite database");

	// Create test table
	execute_sql(
		sqlite_db,
		"CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)",
	)
	.expect("Failed to create table");

	// Insert test data in batches
	execute_sql(sqlite_db, "BEGIN TRANSACTION").expect("Failed to begin transaction");

	for i in 0..row_count {
		let insert_query = format!("INSERT INTO test (value) VALUES ('test_value_{}')", i);
		execute_sql(sqlite_db, &insert_query).expect("Failed to insert data");
	}

	execute_sql(sqlite_db, "COMMIT").expect("Failed to commit transaction");

	sqlite_db
}

struct TestSetup {
	db_name: String,
	row_count: usize,
}

// Setup helper for benchmarks
fn setup_bench(row_count: usize) -> TestSetup {
	// Setup environment
	let (_db, db_name) = setup_sqlite();

	// Setup database
	let sqlite_db = setup_database(&db_name, row_count);

	// Close the database to ensure clean state for benchmarks
	close_sqlite_db(sqlite_db).expect("Failed to close database");

	TestSetup { db_name, row_count }
}

#[divan::bench(args = ROW_COUNTS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn open_database(bencher: Bencher, row_count: usize) {
	bencher
		.with_inputs(|| setup_bench(row_count))
		.bench_values(|setup| {
			// Benchmark opening the database
			let sqlite_db = open_sqlite_db(&setup.db_name, "fdb").expect("Failed to open database");
			close_sqlite_db(sqlite_db).expect("Failed to close database");
		});
}

#[divan::bench(args = ROW_COUNTS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn read_single_row(bencher: Bencher, row_count: usize) {
	bencher
		.with_inputs(|| {
			let setup = setup_bench(row_count);
			let sqlite_db = open_sqlite_db(&setup.db_name, "fdb").expect("Failed to open database");
			(setup, sqlite_db)
		})
		.bench_values(|(setup, sqlite_db)| {
			// Benchmark reading a single row
			let _value = query_count(sqlite_db, "SELECT value FROM test WHERE id = 1")
				.expect("Failed to query");
		});
}

#[divan::bench(args = ROW_COUNTS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn read_all_rows(bencher: Bencher, row_count: usize) {
	bencher
		.with_inputs(|| {
			let setup = setup_bench(row_count);
			let sqlite_db = open_sqlite_db(&setup.db_name, "fdb").expect("Failed to open database");
			(setup, sqlite_db)
		})
		.bench_values(|(setup, sqlite_db)| {
			// Benchmark reading all rows
			let _count =
				query_count(sqlite_db, "SELECT COUNT(*) FROM test").expect("Failed to query");
		});
}

#[divan::bench(args = ROW_COUNTS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn insert_rows(bencher: Bencher, row_count: usize) {
	bencher
		.with_inputs(|| {
			let setup = setup_bench(row_count);
			let sqlite_db = open_sqlite_db(&setup.db_name, "fdb").expect("Failed to open database");
			(setup, sqlite_db)
		})
		.bench_values(|(setup, sqlite_db)| {
			// Benchmark inserting a batch of new rows
			execute_sql(sqlite_db, "BEGIN TRANSACTION").expect("Failed to begin transaction");

			for i in 0..100 {
				let value = setup.row_count + i;
				let insert_query =
					format!("INSERT INTO test (value) VALUES ('new_value_{}')", value);
				execute_sql(sqlite_db, &insert_query).expect("Failed to insert data");
			}

			execute_sql(sqlite_db, "COMMIT").expect("Failed to commit transaction");
		});
}

#[divan::bench(args = ROW_COUNTS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn update_rows(bencher: Bencher, row_count: usize) {
	bencher
		.with_inputs(|| {
			let setup = setup_bench(row_count);
			let sqlite_db = open_sqlite_db(&setup.db_name, "fdb").expect("Failed to open database");
			(setup, sqlite_db)
		})
		.bench_values(|(setup, sqlite_db)| {
			// Benchmark updating existing rows
			execute_sql(sqlite_db, "BEGIN TRANSACTION").expect("Failed to begin transaction");

			// Update first 100 rows or all if fewer
			let limit = if setup.row_count < 100 {
				setup.row_count
			} else {
				100
			};
			let update_query = format!(
				"UPDATE test SET value = 'updated_value' WHERE id <= {}",
				limit
			);
			execute_sql(sqlite_db, &update_query).expect("Failed to update data");

			execute_sql(sqlite_db, "COMMIT").expect("Failed to commit transaction");
		});
}

#[divan::bench(args = ROW_COUNTS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn delete_rows(bencher: Bencher, row_count: usize) {
	bencher
		.with_inputs(|| {
			let setup = setup_bench(row_count);
			let sqlite_db = open_sqlite_db(&setup.db_name, "fdb").expect("Failed to open database");
			(setup, sqlite_db)
		})
		.bench_values(|(setup, sqlite_db)| {
			// Benchmark deleting rows
			execute_sql(sqlite_db, "BEGIN TRANSACTION").expect("Failed to begin transaction");

			// Delete last 100 rows or all if fewer
			let start_id = if setup.row_count > 100 {
				setup.row_count - 100
			} else {
				1
			};
			let delete_query = format!("DELETE FROM test WHERE id >= {}", start_id);
			execute_sql(sqlite_db, &delete_query).expect("Failed to delete data");

			execute_sql(sqlite_db, "COMMIT").expect("Failed to commit transaction");
		});
}

#[divan::bench(args = ROW_COUNTS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn full_lifecycle(bencher: Bencher, row_count: usize) {
	bencher
		.with_inputs(|| setup_bench(row_count))
		.bench_values(|setup| {
			// Open database
			let sqlite_db = open_sqlite_db(&setup.db_name, "fdb").expect("Failed to open database");

			// Read data
			let count =
				query_count(sqlite_db, "SELECT COUNT(*) FROM test").expect("Failed to query count");
			assert!(count > 0, "Database should contain data");

			// Insert new data
			execute_sql(
				sqlite_db,
				"INSERT INTO test (value) VALUES ('lifecycle_test')",
			)
			.expect("Failed to insert");

			// Update data
			execute_sql(
				sqlite_db,
				"UPDATE test SET value = 'updated' WHERE value = 'lifecycle_test'",
			)
			.expect("Failed to update");

			// Delete data
			execute_sql(sqlite_db, "DELETE FROM test WHERE value = 'updated'")
				.expect("Failed to delete");

			// Close database
			close_sqlite_db(sqlite_db).expect("Failed to close database");
		});
}
