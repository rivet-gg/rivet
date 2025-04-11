use lazy_static::lazy_static;
use prometheus::{
    histogram_opts, register_histogram,
    register_histogram_vec, register_int_counter, register_int_counter_vec, register_int_gauge,
    register_int_gauge_vec, Histogram, HistogramVec, IntCounter, IntCounterVec, IntGauge,
    IntGaugeVec,
};
use std::time::Instant;

/// FDB SQLite VFS metrics collection
///
/// This module provides metrics to monitor the performance and behavior of the SQLite VFS
/// implementation with FoundationDB. The metrics are exposed through Prometheus and
/// can be used to monitor, debug, and optimize the system.
///
/// Major categories:
/// - File operations (read, write, etc.)
/// - FoundationDB transactions
/// - Locking behavior
/// - Page management
/// - Error tracking

// Define histogram buckets appropriate for different types of operations
// Latency buckets are in milliseconds
const LATENCY_BUCKETS: &[f64] = &[
    0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0,
    10000.0,
];

// Size buckets in bytes
const SIZE_BUCKETS: &[f64] = &[
    128.0, 256.0, 512.0, 1024.0, 2048.0, 4096.0, 8192.0, 16384.0, 32768.0, 65536.0, 131072.0,
    262144.0, 524288.0, 1048576.0,
];

lazy_static! {
    //
    // File Operations - Read
    //
    pub static ref READ_OPERATIONS_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_read_operations_total",
        "Total number of read operations performed"
    ).unwrap();

    pub static ref READ_BYTES_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_read_bytes_total",
        "Total bytes read from FoundationDB"
    ).unwrap();

    pub static ref READ_OPERATION_ERRORS_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_read_operation_errors_total",
        "Total number of read operation errors"
    ).unwrap();

    pub static ref READ_OPERATION_LATENCY: Histogram = register_histogram!(
        histogram_opts!(
            "sqlite_vfs_fdb_read_operation_latency_milliseconds",
            "Histogram of time taken to complete reads in milliseconds",
            LATENCY_BUCKETS.to_vec()
        )
    ).unwrap();

    pub static ref READ_OPERATIONS_BY_FILE: IntCounterVec = register_int_counter_vec!(
        "sqlite_vfs_fdb_read_operations_by_file",
        "Number of read operations by file",
        &["file_path"]
    ).unwrap();

    //
    // File Operations - Write
    //
    pub static ref WRITE_OPERATIONS_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_write_operations_total",
        "Total number of write operations performed"
    ).unwrap();

    pub static ref WRITE_BYTES_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_write_bytes_total",
        "Total bytes written to FoundationDB"
    ).unwrap();

    pub static ref WRITE_OPERATION_ERRORS_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_write_operation_errors_total",
        "Total number of write operation errors"
    ).unwrap();

    pub static ref WRITE_OPERATION_LATENCY: Histogram = register_histogram!(
        histogram_opts!(
            "sqlite_vfs_fdb_write_operation_latency_milliseconds",
            "Histogram of time taken to complete writes in milliseconds",
            LATENCY_BUCKETS.to_vec()
        )
    ).unwrap();

    pub static ref WRITE_OPERATIONS_BY_FILE: IntCounterVec = register_int_counter_vec!(
        "sqlite_vfs_fdb_write_operations_by_file",
        "Number of write operations by file",
        &["file_path"]
    ).unwrap();

    //
    // File Management
    //
    pub static ref FILE_OPEN_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_file_open_total",
        "Total number of file open operations"
    ).unwrap();

    pub static ref FILE_OPEN_ERRORS_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_file_open_errors_total",
        "Total number of file open errors"
    ).unwrap();

    pub static ref FILE_OPEN_LATENCY: Histogram = register_histogram!(
        histogram_opts!(
            "sqlite_vfs_fdb_file_open_latency_milliseconds",
            "Histogram of time taken to open files in milliseconds",
            LATENCY_BUCKETS.to_vec()
        )
    ).unwrap();

    pub static ref FILE_CLOSE_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_file_close_total",
        "Total number of file close operations"
    ).unwrap();

    pub static ref FILE_DELETE_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_file_delete_total",
        "Total number of file delete operations"
    ).unwrap();

    pub static ref FILE_TRUNCATE_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_file_truncate_total",
        "Total number of file truncate operations"
    ).unwrap();

    pub static ref FILE_SIZE_BYTES: IntGaugeVec = register_int_gauge_vec!(
        "sqlite_vfs_fdb_file_size_bytes",
        "Current size of files in bytes",
        &["file_path"]
    ).unwrap();

    pub static ref OPEN_FILES: IntGauge = register_int_gauge!(
        "sqlite_vfs_fdb_open_files",
        "Current number of open files"
    ).unwrap();

    //
    // FoundationDB Transaction Metrics
    //
    pub static ref FDB_TRANSACTION_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_transaction_total",
        "Total number of FoundationDB transactions"
    ).unwrap();

    pub static ref FDB_TRANSACTION_RETRY_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_transaction_retry_total",
        "Total number of retried FoundationDB transactions"
    ).unwrap();

    pub static ref FDB_TRANSACTION_ERROR_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_transaction_error_total",
        "Total number of FoundationDB transaction errors"
    ).unwrap();

    pub static ref FDB_TRANSACTION_LATENCY: Histogram = register_histogram!(
        histogram_opts!(
            "sqlite_vfs_fdb_transaction_latency_milliseconds",
            "Histogram of time taken to complete FoundationDB transactions in milliseconds",
            LATENCY_BUCKETS.to_vec()
        )
    ).unwrap();

    //
    // Locking and Concurrency Metrics
    //
    pub static ref LOCK_ACQUISITION_TOTAL: IntCounterVec = register_int_counter_vec!(
        "sqlite_vfs_fdb_lock_acquisition_total",
        "Total number of lock acquisitions by lock type",
        &["lock_type"]
    ).unwrap();

    pub static ref LOCK_ESCALATION_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_lock_escalation_total",
        "Total number of lock escalations (upgrades)"
    ).unwrap();

    //
    // Page Management Metrics
    //
    pub static ref PAGE_READ_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_page_read_total",
        "Total number of page reads"
    ).unwrap();

    pub static ref PAGE_WRITE_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_page_write_total",
        "Total number of page writes"
    ).unwrap();

    pub static ref PARTIAL_PAGE_READ_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_partial_page_read_total",
        "Total number of partial page reads"
    ).unwrap();

    pub static ref PARTIAL_PAGE_WRITE_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_partial_page_write_total",
        "Total number of partial page writes"
    ).unwrap();

    pub static ref PAGE_SIZE_BYTES: HistogramVec = register_histogram_vec!(
        histogram_opts!(
            "sqlite_vfs_fdb_page_size_bytes",
            "Histogram of page sizes in bytes",
            SIZE_BUCKETS.to_vec()
        ),
        &["operation"]
    ).unwrap();

    //
    // Metadata Operation Metrics
    //
    pub static ref METADATA_READ_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_metadata_read_total",
        "Total number of metadata read operations"
    ).unwrap();

    pub static ref METADATA_WRITE_TOTAL: IntCounter = register_int_counter!(
        "sqlite_vfs_fdb_metadata_write_total",
        "Total number of metadata write operations"
    ).unwrap();

    pub static ref METADATA_OPERATION_LATENCY: Histogram = register_histogram!(
        histogram_opts!(
            "sqlite_vfs_fdb_metadata_operation_latency_milliseconds",
            "Histogram of time taken to complete metadata operations in milliseconds",
            LATENCY_BUCKETS.to_vec()
        )
    ).unwrap();

    //
    // Error Metrics
    //
    pub static ref VFS_ERROR_TOTAL: IntCounterVec = register_int_counter_vec!(
        "sqlite_vfs_fdb_error_total",
        "Total number of VFS errors by type",
        &["error_type"]
    ).unwrap();
}

/// Timer for measuring operation durations
pub struct MetricsTimer {
    start: Instant,
}

impl MetricsTimer {
    /// Create a new timer starting now
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// End the timer and get the elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        let elapsed = self.start.elapsed();
        (elapsed.as_secs() as f64) * 1000.0 + (elapsed.subsec_nanos() as f64) / 1_000_000.0
    }
}

/// Record read operation metrics
pub fn record_read_operation(file_path: &str, bytes: usize, success: bool) {
    let timer = MetricsTimer::start();
    
    READ_OPERATIONS_TOTAL.inc();
    READ_BYTES_TOTAL.inc_by(bytes as u64);
    READ_OPERATIONS_BY_FILE.with_label_values(&[file_path]).inc();
    
    if !success {
        READ_OPERATION_ERRORS_TOTAL.inc();
    }
    
    READ_OPERATION_LATENCY.observe(timer.elapsed_ms());
}

/// Record write operation metrics
pub fn record_write_operation(file_path: &str, bytes: usize, success: bool) {
    let timer = MetricsTimer::start();
    
    WRITE_OPERATIONS_TOTAL.inc();
    WRITE_BYTES_TOTAL.inc_by(bytes as u64);
    WRITE_OPERATIONS_BY_FILE.with_label_values(&[file_path]).inc();
    
    if !success {
        WRITE_OPERATION_ERRORS_TOTAL.inc();
    }
    
    WRITE_OPERATION_LATENCY.observe(timer.elapsed_ms());
}

/// Start recording file open metrics
pub fn start_file_open() -> MetricsTimer {
    FILE_OPEN_TOTAL.inc();
    MetricsTimer::start()
}

/// Record file open success or failure
pub fn record_file_open_result(file_path: &str, success: bool) {
    if !success {
        FILE_OPEN_ERRORS_TOTAL.inc();
    } else {
        // Increment open file count
        OPEN_FILES.inc();
        // Track per-file metrics
        READ_OPERATIONS_BY_FILE.with_label_values(&[file_path]);
        WRITE_OPERATIONS_BY_FILE.with_label_values(&[file_path]);
    }
}

/// Complete file open latency recording
pub fn complete_file_open(timer: &MetricsTimer) {
    FILE_OPEN_LATENCY.observe(timer.elapsed_ms());
}

/// Record file close metrics
pub fn record_file_close(_file_path: &str, success: bool) {
    FILE_CLOSE_TOTAL.inc();
    
    if success {
        // Decrement the open files counter only on successful close
        OPEN_FILES.dec();
    }
}

/// Record file delete metrics
pub fn record_file_delete(file_path: &str, success: bool) {
    FILE_DELETE_TOTAL.inc();
    
    if success {
        // Remove the file size metric when file is deleted
        FILE_SIZE_BYTES.remove_label_values(&[file_path]).ok();
    }
}

/// Record file truncate metrics
pub fn record_file_truncate(file_path: &str, new_size: i64, success: bool) {
    FILE_TRUNCATE_TOTAL.inc();
    
    if success {
        // Update the file size metric
        FILE_SIZE_BYTES.with_label_values(&[file_path]).set(new_size);
    }
}

/// Record file size update
pub fn update_file_size(file_path: &str, size: i64) {
    FILE_SIZE_BYTES.with_label_values(&[file_path]).set(size);
}

/// Record FoundationDB transaction metrics
pub fn start_fdb_transaction() -> MetricsTimer {
    FDB_TRANSACTION_TOTAL.inc();
    MetricsTimer::start()
}

/// Complete FoundationDB transaction metrics
pub fn complete_fdb_transaction(timer: &MetricsTimer, retries: u64, success: bool) {
    if retries > 0 {
        FDB_TRANSACTION_RETRY_TOTAL.inc_by(retries);
    }
    
    if !success {
        FDB_TRANSACTION_ERROR_TOTAL.inc();
    }
    
    FDB_TRANSACTION_LATENCY.observe(timer.elapsed_ms());
}

/// Record lock acquisition
pub fn record_lock_acquisition(lock_type: &str) {
    LOCK_ACQUISITION_TOTAL.with_label_values(&[lock_type]).inc();
}

/// Record lock escalation
pub fn record_lock_escalation() {
    LOCK_ESCALATION_TOTAL.inc();
}

/// Record page read metrics
pub fn record_page_read(page_size: usize, is_partial: bool) {
    PAGE_READ_TOTAL.inc();
    
    if is_partial {
        PARTIAL_PAGE_READ_TOTAL.inc();
    }
    
    PAGE_SIZE_BYTES.with_label_values(&["read"]).observe(page_size as f64);
}

/// Record page write metrics
pub fn record_page_write(page_size: usize, is_partial: bool) {
    PAGE_WRITE_TOTAL.inc();
    
    if is_partial {
        PARTIAL_PAGE_WRITE_TOTAL.inc();
    }
    
    PAGE_SIZE_BYTES.with_label_values(&["write"]).observe(page_size as f64);
}

/// Record metadata read metrics
pub fn record_metadata_read() -> MetricsTimer {
    METADATA_READ_TOTAL.inc();
    MetricsTimer::start()
}

/// Complete metadata read metrics
pub fn complete_metadata_read(timer: &MetricsTimer) {
    METADATA_OPERATION_LATENCY.observe(timer.elapsed_ms());
}

/// Record metadata write metrics
pub fn record_metadata_write() -> MetricsTimer {
    METADATA_WRITE_TOTAL.inc();
    MetricsTimer::start()
}

/// Complete metadata write metrics
pub fn complete_metadata_write(timer: &MetricsTimer) {
    METADATA_OPERATION_LATENCY.observe(timer.elapsed_ms());
}

/// Record VFS error
pub fn record_vfs_error(error_type: &str) {
    VFS_ERROR_TOTAL.with_label_values(&[error_type]).inc();
}