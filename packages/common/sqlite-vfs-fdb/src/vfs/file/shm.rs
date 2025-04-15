use libsqlite3_sys::*;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::Ordering;
use tracing;

use crate::metrics;
use crate::utils::{SQLITE_IOERR, SQLITE_OK};
use crate::vfs::file::{FdbFile, get_file_ptr};

// Constants for SHM implementation
pub const SHM_INITIAL_SIZE: usize = 32768; // 32KB initial size
pub const SHM_REGION_SIZE: usize = 8192; // 8KB per region (typical SQLite default)
const MAX_SHM_REGIONS: usize = 10; // Maximum number of regions

/// Map a shared memory region
pub unsafe extern "C" fn fdb_shm_map(
    file: *mut sqlite3_file,
    region_index: c_int,      // Which region to map (0..=9)
    region_size: c_int,       // Size of each region in bytes
    is_write: c_int,          // True if writing to the map is allowed
    pp_out: *mut *mut c_void, // OUT: Returns a pointer to the mapped memory
) -> c_int {
    tracing::debug!(
        "fdb_shm_map called: region_index={}, region_size={}, is_write={}",
        region_index,
        region_size,
        is_write
    );

    let fdb_file = match get_file_ptr(file as *mut FdbFile, "fdb_shm_map") {
        Ok(f) => f,
        Err(e) => return e,
    };

    let ext = fdb_file.ext.assume_init_mut();

    // Initialize the SHM buffer if not already done
    if ext.shm_buffer.is_none() {
        // Calculate total buffer size based on maximum possible regions
        let total_size = region_size as usize * MAX_SHM_REGIONS;
        ext.shm_buffer = Some(vec![0u8; total_size]);
        ext.shm_region_size = region_size as usize;
    }

    // Ensure the requested region is valid
    if region_index < 0 || region_index >= MAX_SHM_REGIONS as c_int {
        tracing::error!("Invalid SHM region index: {}", region_index);
        metrics::record_vfs_error("invalid_shm_region");
        return SQLITE_IOERR;
    }

    // Verify the region size is consistent with our allocated buffer
    if region_size as usize != ext.shm_region_size {
        tracing::error!(
            "SHM region size mismatch: requested={}, allocated={}",
            region_size,
            ext.shm_region_size
        );
        metrics::record_vfs_error("shm_region_size_mismatch");
        return SQLITE_IOERR;
    }

    // Calculate the pointer to the start of the requested region
    if let Some(buffer) = ext.shm_buffer.as_mut() {
        let region_offset = region_index as usize * ext.shm_region_size;
        let region_ptr = buffer.as_mut_ptr().add(region_offset);

        // Return the pointer to the mapped region
        *pp_out = region_ptr as *mut c_void;

        // Increment the map count
        ext.shm_map_count += 1;

        SQLITE_OK
    } else {
        tracing::error!("SHM buffer not initialized");
        metrics::record_vfs_error("shm_buffer_not_initialized");
        SQLITE_IOERR
    }
}

/// Lock a shared memory region
pub unsafe extern "C" fn fdb_shm_lock(
    _file: *mut sqlite3_file,
    _offset: c_int, // Offset into the lock array
    _n: c_int,      // Number of slots to lock
    _flags: c_int,  // Flags (SQLITE_SHM_LOCK, SQLITE_SHM_SHARED, etc.)
) -> c_int {
    // For single reader/writer scenario, all locks always succeed
    tracing::debug!("fdb_shm_lock called (always succeeds for single reader/writer)");
    SQLITE_OK
}

/// Memory barrier for shared memory operations
pub unsafe extern "C" fn fdb_shm_barrier(_file: *mut sqlite3_file) {
    // Memory barrier to ensure writes are visible to other processes
    // For single reader/writer, this is essentially a no-op
    tracing::debug!("fdb_shm_barrier called (no-op for single reader/writer)");

    // Issue a compiler memory fence in case of optimizations
    std::sync::atomic::fence(Ordering::SeqCst);
}

/// Unmap a shared memory region
pub unsafe extern "C" fn fdb_shm_unmap(
    file: *mut sqlite3_file,
    delete_flag: c_int, // If true, delete the mapping
) -> c_int {
    tracing::debug!("fdb_shm_unmap called: delete_flag={}", delete_flag);

    let fdb_file = match get_file_ptr(file as *mut FdbFile, "fdb_shm_unmap") {
        Ok(f) => f,
        Err(e) => return e,
    };

    let ext = fdb_file.ext.assume_init_mut();

    // If we don't have a SHM buffer, nothing to do
    if ext.shm_buffer.is_none() {
        return SQLITE_OK;
    }

    // Decrement the map count or delete if requested
    if delete_flag != 0 || ext.shm_map_count <= 1 {
        // Free the SHM buffer
        ext.shm_buffer = None;
        ext.shm_map_count = 0;
    } else {
        // Just decrement the reference count
        ext.shm_map_count -= 1;
    }

    SQLITE_OK
}