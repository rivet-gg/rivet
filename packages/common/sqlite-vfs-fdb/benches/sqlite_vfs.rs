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
use sqlite_vfs_fdb::{close_sqlite_db, execute_sql, open_sqlite_db, query_count};
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

const SAMPLE_SIZE: u32 = 1;
const SAMPLE_COUNT: u32 = 3;
const ARGS: &[(Vfs, usize)] = &[
	// Filesystem variants for baseline comparison
	(Vfs::FsDelete, 10_000),
	(Vfs::FsWal, 10_000),
	(Vfs::FsWal2, 10_000),
	// FoundationDB VFS variants with different compression
	(Vfs::Fdb, 10_000),      // No compression
	(Vfs::FdbLz4, 10_000),   // LZ4 compression
	(Vfs::FdbSnappy, 10_000), // Snappy compression
	(Vfs::FdbZstd, 10_000),  // Zstd compression
];

#[derive(Clone, Copy, Debug)]
enum Vfs {
	// Filesystem with DELETE journal mode
	FsDelete,
	// Filesystem with WAL mode
	FsWal,
	// Filesystem with WAL2 mode
	FsWal2,
	// FoundationDB VFS with no compression
	Fdb,
	// FoundationDB VFS with LZ4 compression
	FdbLz4,
	// FoundationDB VFS with Snappy compression
	FdbSnappy,
	// FoundationDB VFS with Zstd compression
	FdbZstd,
}

impl Vfs {
	fn journal_mode(&self) -> &'static str {
		match self {
			Vfs::FsDelete => "DELETE",
			Vfs::FsWal => "WAL",
			Vfs::FsWal2 => "WAL2",
			Vfs::Fdb => "DELETE",
			Vfs::FdbLz4 => "DELETE",
			Vfs::FdbSnappy => "DELETE",
			Vfs::FdbZstd => "DELETE",
		}
	}

	fn vfs_name(&self) -> &'static str {
		match self {
			// Filesystem VFS types
			Vfs::FsDelete => "unix",
			Vfs::FsWal => "unix",
			Vfs::FsWal2 => "unix",
			// FoundationDB VFS types with different compression algorithms
			Vfs::Fdb => "fdb",            // No compression
			Vfs::FdbLz4 => "fdb_lz4",     // LZ4 compression
			Vfs::FdbSnappy => "fdb_snappy", // Snappy compression
			Vfs::FdbZstd => "fdb_zstd",   // Zstd compression
		}
	}
}

fn main() {
	// Initialize logging only if RUST_LOG is set
	let _ = tracing_subscriber::fmt()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init();

	// Register all VFS variants (None, LZ4, Snappy, Zstd)
	sqlite_vfs_fdb::impls::pages::vfs::register_all_vfs_variants(DATABASE.clone())
		.expect("Failed to register VFS variants");
		
	// Print debug message about VFS registration
	println!("VFS registration complete. Testing registered VFS names:");
	unsafe {
		use libsqlite3_sys::*;
		use std::ffi::CStr;
		
		// List all registered VFSs
		let mut current_vfs = sqlite3_vfs_find(std::ptr::null());
		
		while !current_vfs.is_null() {
			let name = CStr::from_ptr((*current_vfs).zName);
			let name_str = name.to_str().unwrap_or("Invalid UTF-8");
			println!("Found VFS: {}", name_str);
			
			current_vfs = (*current_vfs).pNext;
		}
	}

	// Start the benchmarks
	divan::main();
}

// Generate a unique database name for benchmarks
fn bench_db_name(vfs: Vfs) -> String {
	format!("/tmp/bench_{}_{}.db", vfs.vfs_name(), Uuid::new_v4())
}

// Setup function for the benchmark environment
fn setup_sqlite(vfs: Vfs) -> (Arc<Database>, String) {
	// Setup FoundationDB
	let db = DATABASE.clone();

	// Register all VFS variants (only needs to be called once, but safe to call multiple times)
	sqlite_vfs_fdb::impls::pages::vfs::register_all_vfs_variants(db.clone())
		.expect("Failed to register VFS variants");

	// Generate a unique database name
	let db_name = bench_db_name(vfs);

	(db, db_name)
}

// Helper to set the journal mode for a SQLite database
fn set_journal_mode(sqlite_db: *mut libsqlite3_sys::sqlite3, mode: Vfs) {
	// Only set journal mode for filesystem VFS, not for any FDB VFS variants
	if !matches!(mode, Vfs::Fdb | Vfs::FdbLz4 | Vfs::FdbSnappy | Vfs::FdbZstd) {
		let query = format!("PRAGMA journal_mode={}", mode.journal_mode());
		execute_sql(sqlite_db, &query).expect("Failed to set journal mode");
	}
}

// Helper to create a database with given size and journal mode
fn setup_database(db_name: &str, row_count: usize, vfs: Vfs) -> *mut libsqlite3_sys::sqlite3 {
	// Open the database
	let sqlite_db = open_sqlite_db(db_name, vfs.vfs_name()).expect("Failed to open SQLite database");

	// Set the journal mode
	set_journal_mode(sqlite_db, vfs);

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
	vfs: Vfs,
}

// Setup helper for benchmarks
fn setup_bench(row_count: usize, vfs: Vfs) -> TestSetup {
	// Setup environment
	let (_db, db_name) = setup_sqlite(vfs);

	// Setup database
	let sqlite_db = setup_database(&db_name, row_count, vfs);

	// Close the database to ensure clean state for benchmarks
	close_sqlite_db(sqlite_db).expect("Failed to close database");

	TestSetup {
		db_name,
		row_count,
		vfs,
	}
}

#[divan::bench(args = ARGS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn open_database(bencher: Bencher, args: (Vfs, usize)) {
	let (vfs, row_count) = args;

	bencher
		.with_inputs(|| setup_bench(row_count, vfs))
		.bench_values(|setup| {
			// Benchmark opening the database
			let sqlite_db =
				open_sqlite_db(&setup.db_name, setup.vfs.vfs_name()).expect("Failed to open database");
			set_journal_mode(sqlite_db, setup.vfs);
			close_sqlite_db(sqlite_db).expect("Failed to close database");
		});
}

#[divan::bench(args = ARGS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn read_single_row(bencher: Bencher, args: (Vfs, usize)) {
	let (vfs, row_count) = args;

	bencher
		.with_inputs(|| {
			let setup = setup_bench(row_count, vfs);
			let sqlite_db =
				open_sqlite_db(&setup.db_name, setup.vfs.vfs_name()).expect("Failed to open database");
			set_journal_mode(sqlite_db, setup.vfs);
			(setup, sqlite_db)
		})
		.bench_values(|(_setup, sqlite_db)| {
			// Benchmark reading a single row
			let _value = query_count(sqlite_db, "SELECT value FROM test WHERE id = 1")
				.expect("Failed to query");
		});
}

#[divan::bench(args = ARGS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn read_all_rows(bencher: Bencher, args: (Vfs, usize)) {
	let (vfs, row_count) = args;

	bencher
		.with_inputs(|| {
			let setup = setup_bench(row_count, vfs);
			let sqlite_db =
				open_sqlite_db(&setup.db_name, setup.vfs.vfs_name()).expect("Failed to open database");
			set_journal_mode(sqlite_db, setup.vfs);
			(setup, sqlite_db)
		})
		.bench_values(|(_setup, sqlite_db)| {
			// Benchmark reading all rows
			let _count =
				query_count(sqlite_db, "SELECT COUNT(*) FROM test").expect("Failed to query");
		});
}

#[divan::bench(args = ARGS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn insert_rows(bencher: Bencher, args: (Vfs, usize)) {
	let (vfs, row_count) = args;

	bencher
		.with_inputs(|| {
			let setup = setup_bench(row_count, vfs);
			let sqlite_db =
				open_sqlite_db(&setup.db_name, setup.vfs.vfs_name()).expect("Failed to open database");
			set_journal_mode(sqlite_db, setup.vfs);
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

#[divan::bench(args = ARGS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn update_rows(bencher: Bencher, args: (Vfs, usize)) {
	let (vfs, row_count) = args;

	bencher
		.with_inputs(|| {
			let setup = setup_bench(row_count, vfs);
			let sqlite_db =
				open_sqlite_db(&setup.db_name, setup.vfs.vfs_name()).expect("Failed to open database");
			set_journal_mode(sqlite_db, setup.vfs);
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

#[divan::bench(args = ARGS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn delete_rows(bencher: Bencher, args: (Vfs, usize)) {
	let (vfs, row_count) = args;

	bencher
		.with_inputs(|| {
			let setup = setup_bench(row_count, vfs);
			let sqlite_db =
				open_sqlite_db(&setup.db_name, setup.vfs.vfs_name()).expect("Failed to open database");
			set_journal_mode(sqlite_db, setup.vfs);
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

#[divan::bench(args = ARGS, sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn full_lifecycle(bencher: Bencher, args: (Vfs, usize)) {
	let (vfs, row_count) = args;

	bencher
		.with_inputs(|| setup_bench(row_count, vfs))
		.bench_values(|setup| {
			// Open database
			let sqlite_db =
				open_sqlite_db(&setup.db_name, setup.vfs.vfs_name()).expect("Failed to open database");
			set_journal_mode(sqlite_db, setup.vfs);

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
