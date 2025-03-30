# SQLite VFS for FoundationDB

## Debugging

```
# Build test (for example, file_ops)
RUSTFLAGS="-Zsanitizer=address" cargo test --no-run --message-format=json --test file_ops

# Copy the path of the test that was built

# Run test with lldb (for example, test_file_create_metadata)
RUST_BACKTRACE=1 lldb /Users/nathan/rivet/ee/oss/./target/debug/deps/file_ops-d47459581e41a236 test_file_create_metadata
```

