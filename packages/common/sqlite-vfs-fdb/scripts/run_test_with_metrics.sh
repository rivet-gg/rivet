#!/bin/bash

# Simple script to run a test and show its metrics
# Usage: ./run_test_with_metrics.sh test_name

if [ -z "$1" ]; then
  echo "Usage: ./run_test_with_metrics.sh test_name"
  exit 1
fi

# Run the test
cargo test "$1" -- --nocapture

# Find and display the metrics file
#
# Exclude histograms & unnecesary data
cat target/sqlite_vfs_fdb_metrics_impl_pages_$1.txt | grep -v _bucket | grep -v TYPE | grep -v HELP
