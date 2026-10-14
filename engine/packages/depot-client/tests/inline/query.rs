use super::*;
use libsqlite3_sys::{sqlite3_close, sqlite3_next_stmt, sqlite3_open};

struct MemoryDb(*mut sqlite3);

impl MemoryDb {
	fn open() -> Self {
		let name = CString::new(":memory:").unwrap();
		let mut db = ptr::null_mut();
		let rc = unsafe { sqlite3_open(name.as_ptr(), &mut db) };
		assert_eq!(rc, SQLITE_OK);
		Self(db)
	}

	fn as_ptr(&self) -> *mut sqlite3 {
		self.0
	}
}

impl Drop for MemoryDb {
	fn drop(&mut self) {
		assert_eq!(unsafe { sqlite3_close(self.0) }, SQLITE_OK);
	}
}

#[test]
fn execute_tail_validation_has_no_side_effects_or_statement_leaks() {
	for profiled in [false, true] {
		let db = MemoryDb::open();
		exec_statements(db.as_ptr(), "CREATE TABLE items(value INTEGER)").unwrap();
		for tail in ["PRAGMA query_only=ON", "invalid syntax", "SELECT 2"] {
			let sql = format!("INSERT INTO items VALUES (1); /* trailing */ ; {tail}");
			let result = if profiled {
				execute_single_statement_profiled(db.as_ptr(), &sql, None).map(|(result, _)| result)
			} else {
				execute_single_statement(db.as_ptr(), &sql, None)
			};
			assert!(result.unwrap_err().to_string().contains("single statement"));
			assert!(unsafe { sqlite3_next_stmt(db.as_ptr(), ptr::null_mut()) }.is_null());
			assert_eq!(
				query_statement(db.as_ptr(), "PRAGMA query_only", None)
					.unwrap()
					.rows,
				vec![vec![ColumnValue::Integer(0)]]
			);
			assert_eq!(
				query_statement(db.as_ptr(), "SELECT COUNT(*) FROM items", None)
					.unwrap()
					.rows,
				vec![vec![ColumnValue::Integer(0)]]
			);
		}
		execute_single_statement(db.as_ptr(), "INSERT INTO items VALUES (2)", None).unwrap();
	}
}

#[test]
fn malformed_sql_tail_finalizes_the_first_statement() {
	for profiled in [false, true] {
		let db = MemoryDb::open();
		let result = if profiled {
			execute_single_statement_profiled(db.as_ptr(), "SELECT 1; invalid syntax", None)
				.map(|(result, _)| result)
		} else {
			execute_single_statement(db.as_ptr(), "SELECT 1; invalid syntax", None)
		};
		assert!(result.is_err());
		let outstanding = unsafe { sqlite3_next_stmt(db.as_ptr(), ptr::null_mut()) };
		if !outstanding.is_null() {
			// Clean up a regression's leaked statement before the assertion unwinds.
			unsafe {
				sqlite3_finalize(outstanding);
			}
		}
		assert!(
			outstanding.is_null(),
			"rejected SQL leaked its first statement"
		);
	}
}

#[test]
fn sql_tail_trivia_matches_sqlite_without_preparing_statements() {
	for tail in [
		"",
		" ; ; \t\n\r\u{000b}\u{000c}",
		"-- line comment",
		"/* block */; -- line\n;",
		"/* unterminated",
		"\u{feff};",
	] {
		for profiled in [false, true] {
			let db = MemoryDb::open();
			let sql = format!("SELECT 1;{tail}");
			let expected = exec_statements(db.as_ptr(), &sql).unwrap();
			let actual = if profiled {
				execute_single_statement_profiled(db.as_ptr(), &sql, None).map(|(result, _)| result)
			} else {
				execute_single_statement(db.as_ptr(), &sql, None)
			}
			.unwrap();
			assert_eq!(actual.rows, expected.rows, "tail: {tail:?}");
		}
	}
}

#[test]
fn transaction_exec_tail_validation_does_not_prepare_after_terminator() {
	for profiled in [false, true] {
		for terminator in ["COMMIT", "ROLLBACK"] {
			let db = MemoryDb::open();
			exec_statements(db.as_ptr(), "CREATE TABLE items(value INTEGER); BEGIN").unwrap();
			let sql = format!("{terminator}; PRAGMA query_only=ON");
			let result = if profiled {
				exec_statements_profiled_in_transaction(db.as_ptr(), &sql).map(|(result, _)| result)
			} else {
				exec_statements_in_transaction(db.as_ptr(), &sql)
			};
			assert!(result.unwrap_err().is::<SqliteTransactionClosedError>());
			assert!(unsafe { sqlite3_next_stmt(db.as_ptr(), ptr::null_mut()) }.is_null());
			execute_single_statement(db.as_ptr(), "INSERT INTO items VALUES (1)", None).unwrap();
		}
	}
}

#[test]
fn run_and_query_bind_typed_params() {
	let db = MemoryDb::open();
	exec_statements(
		db.as_ptr(),
		"CREATE TABLE items(id INTEGER PRIMARY KEY, label TEXT, score REAL, payload BLOB);",
	)
	.unwrap();

	let result = execute_statement(
		db.as_ptr(),
		"INSERT INTO items(label, score, payload) VALUES (?, ?, ?);",
		Some(&[
			BindParam::Text("alpha".to_owned()),
			BindParam::Float(3.5),
			BindParam::Blob(vec![1, 2, 3]),
		]),
	)
	.unwrap();
	assert_eq!(result.changes, 1);

	let rows = query_statement(
		db.as_ptr(),
		"SELECT id, label, score, payload FROM items WHERE label = ?;",
		Some(&[BindParam::Text("alpha".to_owned())]),
	)
	.unwrap();
	assert_eq!(rows.columns, vec!["id", "label", "score", "payload"]);
	assert_eq!(
		rows.rows,
		vec![vec![
			ColumnValue::Integer(1),
			ColumnValue::Text("alpha".to_owned()),
			ColumnValue::Float(3.5),
			ColumnValue::Blob(vec![1, 2, 3]),
		]]
	);
}

#[test]
fn exec_returns_last_statement_rows() {
	let db = MemoryDb::open();
	let result = exec_statements(
		db.as_ptr(),
		"CREATE TABLE items(id INTEGER); INSERT INTO items VALUES (1), (2); SELECT COUNT(*) AS count FROM items;",
	)
	.unwrap();

	assert_eq!(result.columns, vec!["count"]);
	assert_eq!(result.rows, vec![vec![ColumnValue::Integer(2)]]);
}

#[test]
fn exec_keeps_only_the_last_compatible_result_set() {
	let db = MemoryDb::open();
	let result = exec_statements(db.as_ptr(), "SELECT 1 AS value; SELECT 2 AS value;")
		.expect("both selects should execute");

	assert_eq!(result.columns, vec!["value"]);
	assert_eq!(result.rows, vec![vec![ColumnValue::Integer(2)]]);
}

#[test]
fn transaction_exec_allows_comments_after_commit() {
	let db = MemoryDb::open();
	exec_statements(db.as_ptr(), "BEGIN").unwrap();
	let result = exec_statements_in_transaction(db.as_ptr(), "COMMIT; -- complete")
		.expect("a trailing comment is not another transaction statement");
	assert_eq!(result.readonly, Some(true));
}

#[test]
fn exec_reports_the_actual_failing_statement_index() {
	let db = MemoryDb::open();
	exec_statements(
		db.as_ptr(),
		"CREATE TABLE unique_items(value INTEGER UNIQUE);",
	)
	.unwrap();

	let error = exec_statements(
		db.as_ptr(),
		"INSERT INTO unique_items VALUES (1); INSERT INTO unique_items VALUES (1);",
	)
	.expect_err("the second statement should violate the unique constraint");
	let typed = error.downcast_ref::<SqliteStatementError>().unwrap();
	assert_eq!(typed.code, libsqlite3_sys::SQLITE_CONSTRAINT_UNIQUE);
	assert_eq!(typed.statement_index, 1);
}

#[test]
fn execute_single_statement_returns_rows_and_metadata() {
	let db = MemoryDb::open();
	let result = execute_single_statement(db.as_ptr(), "SELECT 7 AS value;", None).unwrap();

	assert_eq!(result.columns, vec!["value"]);
	assert_eq!(result.rows, vec![vec![ColumnValue::Integer(7)]]);
	assert_eq!(result.changes, 0);
	assert_eq!(result.last_insert_row_id, None);
}

#[test]
fn profiled_execute_counts_bind_and_result_logical_bytes_during_conversion() {
	let db = MemoryDb::open();
	let (result, profile) = execute_single_statement_profiled(
		db.as_ptr(),
		"SELECT ?, ?, ?;",
		Some(&[
			BindParam::Integer(7),
			BindParam::Text("hello".to_owned()),
			BindParam::Blob(vec![1, 2, 3]),
		]),
	)
	.unwrap();

	assert_eq!(result.rows.len(), 1);
	assert_eq!(profile.bind_count, 3);
	assert_eq!(profile.bind_logical_bytes, 16);
	assert_eq!(profile.result_rows, 1);
	assert_eq!(profile.result_columns, 3);
	assert_eq!(profile.result_logical_bytes, 16);
}

#[test]
fn execute_single_statement_returns_write_metadata() {
	let db = MemoryDb::open();
	exec_statements(
		db.as_ptr(),
		"CREATE TABLE execute_items(id INTEGER PRIMARY KEY, label TEXT);",
	)
	.unwrap();

	let result = execute_single_statement(
		db.as_ptr(),
		"INSERT INTO execute_items(label) VALUES (?);",
		Some(&[BindParam::Text("alpha".to_owned())]),
	)
	.unwrap();

	assert_eq!(result.columns, Vec::<String>::new());
	assert_eq!(result.rows, Vec::<Vec<ColumnValue>>::new());
	assert_eq!(result.changes, 1);
	assert_eq!(result.last_insert_row_id, Some(1));
}

#[test]
fn execute_single_statement_collects_insert_returning_rows() {
	let db = MemoryDb::open();
	exec_statements(
		db.as_ptr(),
		"CREATE TABLE execute_returning(id INTEGER PRIMARY KEY, label TEXT);",
	)
	.unwrap();

	let result = execute_single_statement(
		db.as_ptr(),
		"INSERT INTO execute_returning(label) VALUES ('bravo') RETURNING id, label;",
		None,
	)
	.unwrap();

	assert_eq!(result.columns, vec!["id", "label"]);
	assert_eq!(
		result.rows,
		vec![vec![
			ColumnValue::Integer(1),
			ColumnValue::Text("bravo".to_owned())
		]]
	);
	assert_eq!(result.changes, 1);
	assert_eq!(result.last_insert_row_id, Some(1));
}

#[test]
fn execute_single_statement_collects_readonly_pragma_rows() {
	let db = MemoryDb::open();
	let result = execute_single_statement(db.as_ptr(), "PRAGMA user_version;", None).unwrap();

	assert_eq!(result.columns, vec!["user_version"]);
	assert_eq!(result.rows, vec![vec![ColumnValue::Integer(0)]]);
	assert_eq!(result.changes, 0);
}

#[test]
fn execute_single_statement_runs_mutating_pragma() {
	let db = MemoryDb::open();
	let result = execute_single_statement(db.as_ptr(), "PRAGMA user_version = 9;", None).unwrap();

	assert_eq!(result.columns, Vec::<String>::new());
	assert_eq!(result.rows, Vec::<Vec<ColumnValue>>::new());

	let version = execute_single_statement(db.as_ptr(), "PRAGMA user_version;", None).unwrap();
	assert_eq!(version.rows, vec![vec![ColumnValue::Integer(9)]]);
}

#[test]
fn execute_single_statement_rejects_multi_statement_sql() {
	let db = MemoryDb::open();
	let err = execute_single_statement(db.as_ptr(), "SELECT 1; SELECT 2;", None)
		.expect_err("multi statement execute should fail");

	assert!(
		err.to_string().contains("single statement"),
		"unexpected error: {err:#}"
	);
	let typed = err.downcast_ref::<SqliteStatementError>().unwrap();
	assert_eq!(typed.code, -1);
	assert_eq!(typed.statement_index, 0);
}

#[test]
fn execute_single_statement_reports_extended_result_code() {
	let db = MemoryDb::open();
	exec_statements(
		db.as_ptr(),
		"CREATE TABLE unique_items(value TEXT UNIQUE); INSERT INTO unique_items VALUES ('same');",
	)
	.unwrap();
	let err = execute_single_statement(
		db.as_ptr(),
		"INSERT INTO unique_items VALUES ('same')",
		None,
	)
	.expect_err("unique constraint should fail");
	let typed = err.downcast_ref::<SqliteStatementError>().unwrap();
	assert_eq!(typed.code, libsqlite3_sys::SQLITE_CONSTRAINT_UNIQUE);
}

#[test]
fn execute_single_statement_reports_malformed_sql() {
	let db = MemoryDb::open();
	let err = execute_single_statement(db.as_ptr(), "SELECT FROM", None)
		.expect_err("malformed execute should fail");

	assert!(
		err.to_string().contains("failed to prepare"),
		"unexpected error: {err:#}"
	);
}
