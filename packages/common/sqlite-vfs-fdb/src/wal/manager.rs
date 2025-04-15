use crate::fdb::keyspace::FdbKeySpace;
use crate::utils::{run_fdb_tx, FdbVfsError};
use crate::wal::parser::{WalFrame, WalHeader, WalParser};
use foundationdb::{Database, FdbBindingError};
use std::sync::Arc;
use uuid::Uuid;

/// Manages WAL file operations for the FoundationDB VFS
pub struct WalManager {
    db: Arc<Database>,
    keyspace: FdbKeySpace,
}

impl WalManager {
    /// Create a new WAL manager
    pub fn new(db: Arc<Database>, keyspace: FdbKeySpace) -> Self {
        Self { db, keyspace }
    }

    /// Store the WAL header in FDB
    pub fn store_wal_header(&self, file_id: &Uuid, header: &WalHeader) -> Result<(), FdbVfsError> {
        let header_key = self.keyspace.wal_header_key(file_id);
        let header_bytes = header.to_bytes();

        let db = self.db.clone();
        let header_key_clone = header_key.clone();
        let header_bytes_clone = header_bytes.clone();

        run_fdb_tx(&db, move |tx| {
            let header_key = header_key_clone.clone();
            let header_bytes = header_bytes_clone.clone();

            async move {
                tx.set(&header_key, &header_bytes);
                Ok(())
            }
        })
        .map_err(|e| FdbVfsError::Other(format!("Failed to store WAL header: {}", e)))
    }

    /// Get the WAL header from FDB
    pub fn get_wal_header(&self, file_id: &Uuid) -> Result<Option<WalHeader>, FdbVfsError> {
        let header_key = self.keyspace.wal_header_key(file_id);
        let db = self.db.clone();
        
        let result = run_fdb_tx(&db, move |tx| {
            let header_key_clone = header_key.clone();
            
            async move {
                let result = tx.get(&header_key_clone, false).await?;
                
                if let Some(bytes) = result {
                    match WalHeader::from_bytes(&bytes) {
                        Ok(header) => Ok(Some(header)),
                        Err(e) => {
                            // Use a binding error
                            let error_message = format!("Failed to parse WAL header: {}", e);
                            tracing::error!("{}", error_message);
                            let err_box = Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_message));
                            Err(FdbBindingError::new_custom_error(err_box))
                        }
                    }
                } else {
                    Ok(None)
                }
            }
        });
        
        match result {
            Ok(header) => Ok(header),
            Err(e) => Err(FdbVfsError::Other(format!("Failed to get WAL header: {}", e)))
        }
    }

    /// Store a WAL frame in FDB
    pub fn store_wal_frame(&self, file_id: &Uuid, frame: &WalFrame, frame_idx: u32) -> Result<(), FdbVfsError> {
        let frame_key = self.keyspace.wal_frame_key(file_id, frame.salt_1, frame.salt_2, frame_idx);
        
        // Serialize the frame header and page data
        let mut frame_data = frame.header_to_bytes();
        frame_data.extend_from_slice(&frame.page_data);
        
        let db = self.db.clone();
        let frame_key_clone = frame_key.clone();
        let frame_data_clone = frame_data.clone();
        
        run_fdb_tx(&db, move |tx| {
            let frame_key = frame_key_clone.clone();
            let frame_data = frame_data_clone.clone();
            
            async move {
                tx.set(&frame_key, &frame_data);
                Ok(())
            }
        })
        .map_err(|e| FdbVfsError::Other(format!("Failed to store WAL frame: {}", e)))
    }

    /// Process and store WAL data from a write operation
    pub fn process_wal_write(&self, file_id: &Uuid, offset: i64, data: &[u8], _page_size: usize) -> Result<usize, FdbVfsError> {
        tracing::debug!("Processing WAL write: offset={}, data_len={}", offset, data.len());

        // If the write is to offset 0, it might include the WAL header
        if offset == 0 {
            if data.len() >= super::parser::WAL_HEADER_SIZE {
                // Try to parse and store the header
                match WalHeader::from_bytes(&data[0..super::parser::WAL_HEADER_SIZE]) {
                    Ok(header) => {
                        self.store_wal_header(file_id, &header)?;
                        
                        // If the data only contains the header, we're done
                        if data.len() == super::parser::WAL_HEADER_SIZE {
                            return Ok(super::parser::WAL_HEADER_SIZE);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse WAL header: {}", e);
                        // Continue anyway, as SQLite might be writing partial data
                    }
                }
            }
        }
        
        // If we're not writing at the beginning, or we have more data than just the header
        // try to parse frames from the data
        let mut parser = WalParser::new();
        let mut frames_written = 0;
        
        // We need the header to get the page size
        let header = match self.get_wal_header(file_id)? {
            Some(h) => h,
            None => {
                // If we don't have a header yet, we can't parse frames
                // Just store raw bytes for now and wait for a complete header
                tracing::warn!("No WAL header found, can't parse frames");
                return Ok(data.len());
            }
        };
        
        // Skip header if it's included in the data and we're at offset 0
        let data_to_parse = if offset == 0 && data.len() >= super::parser::WAL_HEADER_SIZE {
            &data[super::parser::WAL_HEADER_SIZE..]
        } else {
            data
        };
        
        // Add data to the parser
        parser.add_data(data_to_parse);
        
        // Calculate the frame index based on the offset
        let frame_size = super::parser::FRAME_HEADER_SIZE + header.page_size as usize;
        let first_frame_idx = if offset == 0 {
            0
        } else {
            // If offset > header size, calculate frame index
            ((offset - super::parser::WAL_HEADER_SIZE as i64) / frame_size as i64) as u32
        };
        
        // Process frames
        let bytes_processed = match parser.process(|frame| {
            let frame_idx = first_frame_idx + frames_written;
            if let Err(e) = self.store_wal_frame(file_id, &frame, frame_idx) {
                tracing::error!("Failed to store WAL frame {}: {}", frame_idx, e);
            }
            frames_written += 1;
        }) {
            Ok(count) => {
                tracing::debug!("Processed {} WAL frames", count);
                data_to_parse.len()
            }
            Err(e) => {
                tracing::error!("Error processing WAL frames: {}", e);
                0
            }
        };
        
        // Return total bytes processed including header if present
        if offset == 0 && data.len() >= super::parser::WAL_HEADER_SIZE {
            Ok(super::parser::WAL_HEADER_SIZE + bytes_processed)
        } else {
            Ok(bytes_processed)
        }
    }
    
    /// Read WAL data for a read operation
    pub fn read_wal_data(&self, file_id: &Uuid, offset: i64, count: usize) -> Result<Vec<u8>, FdbVfsError> {
        tracing::debug!("Reading WAL data: offset={}, count={}", offset, count);
        
        // Create buffer for the result
        let mut result = vec![0u8; count];
        
        // If the read includes the WAL header
        if offset < super::parser::WAL_HEADER_SIZE as i64 {
            // Get the header
            let header = match self.get_wal_header(file_id)? {
                Some(h) => h,
                None => {
                    // If no header exists yet, return zeros
                    return Ok(result);
                }
            };
            
            // Serialize the header
            let header_bytes = header.to_bytes();
            
            // Calculate how much of the header to copy
            let header_start = offset as usize;
            let header_count = std::cmp::min(
                super::parser::WAL_HEADER_SIZE - header_start,
                count
            );
            
            // Copy header data to result
            if header_start < header_bytes.len() {
                let end = std::cmp::min(header_start + header_count, header_bytes.len());
                result[0..end - header_start].copy_from_slice(&header_bytes[header_start..end]);
            }
            
            // If we've filled the buffer, return it
            if header_count == count {
                return Ok(result);
            }
            
            // Otherwise, continue to read frame data
        }
        
        // Not implemented yet - for now, just return zeros
        // In a complete implementation, we would:
        // 1. Determine which frame(s) the read spans
        // 2. Retrieve those frames from FDB
        // 3. Copy the relevant portions to the result buffer
        
        Ok(result)
    }
}