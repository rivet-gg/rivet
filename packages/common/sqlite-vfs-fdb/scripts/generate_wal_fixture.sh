#!/usr/bin/env bash
set -euo pipefail

# Create fixtures directory
FIXTURE_DIR="tests/fixtures"
mkdir -p "$FIXTURE_DIR"

# Database paths
DB_PATH="$FIXTURE_DIR/test_wal.db"
WAL_PATH="$FIXTURE_DIR/test_wal.db-wal"

# Remove old files if they exist
rm -f "$DB_PATH" "$WAL_PATH" "$DB_PATH-shm"

# Create database with WAL mode by piping commands to SQLite REPL
echo "Creating database in WAL mode..."
sqlite3 "$DB_PATH" <<EOF
PRAGMA journal_mode = WAL;
PRAGMA wal_autocheckpoint = 2147483647;
EOF

sqlite3 "$DB_PATH" <<EOF
CREATE TABLE test_table (id INTEGER PRIMARY KEY, data TEXT);
INSERT INTO test_table VALUES (1, 'first row');
INSERT INTO test_table VALUES (2, 'second row');
EOF

# Show file sizes
echo "Database file size: $(stat -f "%z" "$DB_PATH") bytes"
echo "WAL file size: $(stat -f "%z" "$WAL_PATH") bytes"
