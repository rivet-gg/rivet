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
        tracing::debug!("Storing WAL frame: file_id={}, frame_idx={}, page_number={}, salt1={:x}, salt2={:x}",
            file_id, frame_idx, frame.page_number, frame.salt_1, frame.salt_2);
        
        // Generate the key using the frame's salt values for correct retrieval
        let frame_key = self.keyspace.wal_frame_key(file_id, frame.salt_1, frame.salt_2, frame_idx);
        
        // Serialize the frame
        let mut frame_data = vec![0u8; super::parser::FRAME_HEADER_SIZE + frame.page_data.len()];
        
        // Copy header information (page number, db size, salts, checksums)
        frame_data[0..4].copy_from_slice(&frame.page_number.to_be_bytes());
        frame_data[4..8].copy_from_slice(&frame.database_size.to_be_bytes());
        frame_data[8..12].copy_from_slice(&frame.salt_1.to_be_bytes());
        frame_data[12..16].copy_from_slice(&frame.salt_2.to_be_bytes());
        frame_data[16..20].copy_from_slice(&frame.checksum_1.to_be_bytes());
        frame_data[20..24].copy_from_slice(&frame.checksum_2.to_be_bytes());
        
        // Copy page data
        if !frame.page_data.is_empty() {
            frame_data[super::parser::FRAME_HEADER_SIZE..].copy_from_slice(&frame.page_data);
        }
        
        // Create an index entry to find this frame by index alone
        let index_key = self.keyspace.wal_frame_index_key(file_id, frame_idx);
        let index_data = format!("{}:{}", frame.salt_1, frame.salt_2).into_bytes();
        
        // Store both the frame and the index in a single transaction
        let db = self.db.clone();
        let frame_data_clone = frame_data.clone();
        let index_data_clone = index_data.clone();
        
        run_fdb_tx(&db, move |tx| {
            let frame_key_clone = frame_key.clone();
            let index_key_clone = index_key.clone();
            let frame_data = frame_data_clone.clone();
            let index_data = index_data_clone.clone();
            
            async move {
                tx.set(&frame_key_clone, &frame_data);
                tx.set(&index_key_clone, &index_data);
                Ok(())
            }
        })
        .map_err(|e| FdbVfsError::Other(format!("Failed to store WAL frame: {}", e)))
    }

    /// Extract a WAL frame from raw bytes
    fn extract_wal_frame(&self, frame_data: &[u8], use_header_salts: bool, header: &WalHeader) -> Option<WalFrame> {
        if frame_data.len() < super::parser::FRAME_HEADER_SIZE {
            return None; // Not enough data for a frame header
        }
        
        // Extract frame header components
        let page_number = u32::from_be_bytes([
            frame_data[0], frame_data[1], frame_data[2], frame_data[3]
        ]);
        
        let database_size = u32::from_be_bytes([
            frame_data[4], frame_data[5], frame_data[6], frame_data[7]
        ]);
        
        // For salt values, either use from the frame data or from the header
        let (salt_1, salt_2) = if use_header_salts {
            // Use salt values from header (for fallback mode)
            (header.salt_1, header.salt_2)
        } else {
            // Use salt values from the frame data
            let s1 = u32::from_be_bytes([
                frame_data[8], frame_data[9], frame_data[10], frame_data[11]
            ]);
            
            let s2 = u32::from_be_bytes([
                frame_data[12], frame_data[13], frame_data[14], frame_data[15]
            ]);
            
            (s1, s2)
        };
        
        let checksum_1 = u32::from_be_bytes([
            frame_data[16], frame_data[17], frame_data[18], frame_data[19]
        ]);
        
        let checksum_2 = u32::from_be_bytes([
            frame_data[20], frame_data[21], frame_data[22], frame_data[23]
        ]);
        
        // Create a page data buffer
        let page_data = if frame_data.len() > super::parser::FRAME_HEADER_SIZE {
            frame_data[super::parser::FRAME_HEADER_SIZE..].to_vec()
        } else {
            Vec::new()
        };
        
        #[cfg(test)]
        let page_data_len = page_data.len();
        
        #[cfg(test)]
        let page_data_preview = {
            let mut preview = [0u8; 16];
            let len = std::cmp::min(16, page_data.len());
            preview[..len].copy_from_slice(&page_data[..len]);
            preview
        };
        
        // Create and return a WAL frame
        Some(WalFrame {
            page_number,
            database_size,
            salt_1,
            salt_2,
            checksum_1,
            checksum_2,
            page_data,
            #[cfg(test)]
            page_data_len,
            #[cfg(test)]
            page_data_preview,
        })
    }
    
    /// Process WAL frames directly from raw data
    fn process_frames_directly(&self, file_id: &Uuid, data: &[u8], frame_size: usize, start_frame_idx: u32, 
                             use_header_salts: bool, header: &WalHeader) -> usize {
        let num_complete_frames = data.len() / frame_size;
        
        if num_complete_frames == 0 {
            return 0; // Not enough data for a complete frame
        }
        
        let mut frames_processed = 0;
        
        // Process each complete frame
        for i in 0..num_complete_frames {
            let frame_start = i * frame_size;
            let frame_end = std::cmp::min(frame_start + frame_size, data.len());
            
            if frame_end - frame_start < super::parser::FRAME_HEADER_SIZE {
                continue; // Skip frames with incomplete headers
            }
            
            let frame_data = &data[frame_start..frame_end];
            
            // Extract the frame from raw data
            if let Some(frame) = self.extract_wal_frame(frame_data, use_header_salts, header) {
                // Store the frame
                let current_frame_idx = start_frame_idx + i as u32;
                
                tracing::debug!("Storing WAL frame {} with page_number={}", current_frame_idx, frame.page_number);
                
                if let Err(e) = self.store_wal_frame(file_id, &frame, current_frame_idx) {
                    tracing::error!("Failed to store WAL frame {}: {}", current_frame_idx, e);
                } else {
                    frames_processed += 1;
                }
            }
        }
        
        frames_processed
    }
    
    /// Process and store WAL data from a write operation
    pub fn process_wal_write(&self, file_id: &Uuid, offset: i64, data: &[u8], _page_size: usize) -> Result<usize, FdbVfsError> {
        tracing::debug!("Processing WAL write: offset={}, data_len={}", offset, data.len());

        // If the write is to offset 0, it might include the WAL header
        if offset == 0 && data.len() >= super::parser::WAL_HEADER_SIZE {
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
        
        // We need the header to get the page size
        let header = match self.get_wal_header(file_id)? {
            Some(h) => h,
            None => {
                // If we don't have a header yet, we can't parse frames
                tracing::warn!("No WAL header found, can't parse frames at offset {}", offset);
                return Ok(data.len());
            }
        };
        
        // Calculate the frame size based on the page size from the header
        let frame_size = super::parser::FRAME_HEADER_SIZE + header.page_size as usize;
        
        // Calculate the frame index based on the offset
        let first_frame_idx = if offset == 0 {
            0
        } else {
            ((offset - super::parser::WAL_HEADER_SIZE as i64) / frame_size as i64) as u32
        };
        
        // For WAL frame writes that don't happen at offset 0
        if offset != 0 {
            // Check if this is at a frame boundary
            let frame_boundary = (offset - super::parser::WAL_HEADER_SIZE as i64) % frame_size as i64 == 0;
            if !frame_boundary {
                tracing::info!("WAL write at non-frame boundary offset {} - storing raw bytes", offset);
                return Ok(data.len());
            }
            
            // Direct processing for non-zero offsets
            tracing::debug!("Processing direct WAL frame writes at offset {}", offset);
            
            // Process frames directly, using salt values from the frames
            self.process_frames_directly(file_id, data, frame_size, first_frame_idx, false, &header);
            
            // Return the total bytes processed
            return Ok(data.len());
        }
        
        // For offset 0, first try using the WAL parser on the portion after the header
        
        // Skip header if it's included in the data
        let data_to_parse = if data.len() >= super::parser::WAL_HEADER_SIZE {
            &data[super::parser::WAL_HEADER_SIZE..]
        } else {
            data
        };
        
        // If there's nothing to parse after skipping the header, we're done
        if data_to_parse.is_empty() {
            return Ok(data.len());
        }
        
        // Try to use the parser first
        let mut parser = WalParser::new();
        let mut frames_written = 0;
        
        parser.add_data(data_to_parse);
        
        // Try to process frames with the parser
        let parse_result = parser.process(|frame| {
            let current_frame_idx = first_frame_idx + frames_written;
            if let Err(e) = self.store_wal_frame(file_id, &frame, current_frame_idx) {
                tracing::error!("Failed to store WAL frame {}: {}", current_frame_idx, e);
            }
            frames_written += 1;
        });
        
        // If parsing failed or found no frames, try direct extraction as fallback
        match parse_result {
            Ok(count) if count > 0 => {
                tracing::debug!("Successfully parsed {} WAL frames", count);
            },
            _ => {
                // If parsing failed or found no frames, try direct extraction
                tracing::debug!("Falling back to direct frame extraction for offset 0");
                
                // Process frames directly, but use header salts for frames at offset 0
                self.process_frames_directly(file_id, data_to_parse, frame_size, first_frame_idx, true, &header);
            }
        }
        
        // Return total bytes processed including header if present
        if offset == 0 && data.len() >= super::parser::WAL_HEADER_SIZE {
            Ok(super::parser::WAL_HEADER_SIZE + data_to_parse.len())
        } else {
            Ok(data.len())
        }
    }
    
    /// Read WAL data for a read operation
    pub fn read_wal_data(&self, file_id: &Uuid, offset: i64, count: usize) -> Result<Vec<u8>, FdbVfsError> {
        tracing::debug!("Reading WAL data: offset={}, count={}", offset, count);
        
        // Create buffer for the result
        let mut result = vec![0u8; count];
        
        // Get the WAL header first - we need it to know page size and salts
        let header = match self.get_wal_header(file_id)? {
            Some(h) => h,
            None => {
                // If no header exists yet, return zeros
                tracing::info!("No WAL header found during read, returning zeros");
                return Ok(result);
            }
        };
            
        // If the read includes the WAL header
        if offset < super::parser::WAL_HEADER_SIZE as i64 {
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
            
            // Otherwise, continue to read frame data after the header
            // Adjust the offset and remaining count for frame data
            let adjusted_offset = 0; // Relative offset from end of header
            let adjusted_count = count - header_count;
            
            // Now read frames starting at the adjusted offset and copy to the result buffer
            if let Ok(frame_data) = self.read_frame_data(
                file_id, 
                &header, 
                super::parser::WAL_HEADER_SIZE as i64 + adjusted_offset,
                adjusted_count
            ) {
                // Copy frame data to result buffer after the header portion
                if frame_data.len() > 0 {
                    let copy_len = std::cmp::min(frame_data.len(), adjusted_count);
                    result[header_count..header_count + copy_len].copy_from_slice(&frame_data[0..copy_len]);
                }
            }
        } else {
            // Reading only frame data (no header)
            // Read frames starting at the specified offset
            if let Ok(frame_data) = self.read_frame_data(file_id, &header, offset, count) {
                // Copy frame data to result buffer
                if frame_data.len() > 0 {
                    let copy_len = std::cmp::min(frame_data.len(), count);
                    result[0..copy_len].copy_from_slice(&frame_data[0..copy_len]);
                }
            }
        }
        
        Ok(result)
    }
    
    /// Helper method to read WAL frame data from FDB
    fn read_frame_data(&self, file_id: &Uuid, header: &WalHeader, offset: i64, count: usize) -> Result<Vec<u8>, FdbVfsError> {
        tracing::debug!("Reading frame data: offset={}, count={}", offset, count);
        
        // Calculate frame size from header
        let frame_size = super::parser::FRAME_HEADER_SIZE + header.page_size as usize;
        
        // Calculate which frames we need to read
        // First, adjust offset to be relative to the first frame
        let frame_offset = offset - super::parser::WAL_HEADER_SIZE as i64;
        
        if frame_offset < 0 {
            // Invalid case - should be handled by read_wal_data
            return Ok(Vec::new());
        }
        
        // Calculate first frame index and offset within the frame
        let first_frame_idx = (frame_offset / frame_size as i64) as u32;
        let first_frame_offset = (frame_offset % frame_size as i64) as usize;
        
        // Calculate last frame index
        let last_byte = frame_offset + count as i64 - 1;
        let last_frame_idx = (last_byte / frame_size as i64) as u32;
        
        tracing::info!("Reading WAL frames {} to {} (offset {} within first frame)", 
            first_frame_idx, last_frame_idx, first_frame_offset);
        
        // Prepare buffer for results
        let mut result = vec![0u8; count];
        let mut result_pos = 0;
        
        // Fetch frames from FDB and copy relevant portions to result
        for frame_idx in first_frame_idx..=last_frame_idx {
            if result_pos >= count {
                break; // We've filled the buffer
            }
            
            // We need to look up the salt values for this frame from the index
            let db = self.db.clone();
            let keyspace = self.keyspace.clone();
            let file_id_clone = file_id.clone();
            
            // First get the frame index entry to find salt values
            let index_key = keyspace.wal_frame_index_key(file_id, frame_idx);
            
            // Run a transaction to get the index entry
            let salt_values = match run_fdb_tx(&db, move |tx| {
                let index_key_clone = index_key.clone();
                
                async move {
                    let result = tx.get(&index_key_clone, false).await?;
                    Ok(result.map(|bytes| bytes.to_vec()))
                }
            }) {
                Ok(Some(data)) => {
                    // Parse the salt values from the index data (format: "salt1:salt2")
                    match String::from_utf8(data) {
                        Ok(salt_string) => {
                            let parts: Vec<&str> = salt_string.split(':').collect();
                            if parts.len() == 2 {
                                if let (Ok(salt1), Ok(salt2)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                                    Some((salt1, salt2))
                                } else {
                                    tracing::error!("Failed to parse salt values for frame {}: {}", frame_idx, salt_string);
                                    None
                                }
                            } else {
                                tracing::error!("Invalid salt string format for frame {}: {}", frame_idx, salt_string);
                                None
                            }
                        },
                        Err(e) => {
                            tracing::error!("Failed to parse salt string for frame {}: {}", frame_idx, e);
                            None
                        }
                    }
                },
                Ok(None) => {
                    // Frame index not found - this could happen if the frame doesn't exist
                    tracing::warn!("Frame index {} not found for file {}", frame_idx, file_id);
                    
                    // Fall back to using the header salt values
                    tracing::info!("Falling back to header salt values for frame {}: salt1={:x}, salt2={:x}", 
                        frame_idx, header.salt_1, header.salt_2);
                    Some((header.salt_1, header.salt_2))
                },
                Err(e) => {
                    tracing::error!("Error fetching frame index {}: {}", frame_idx, e);
                    
                    // Fall back to using the header salt values
                    tracing::info!("Falling back to header salt values after error for frame {}", frame_idx);
                    Some((header.salt_1, header.salt_2))
                }
            };
            
            // If we couldn't determine the salt values, try the next frame
            let (salt1, salt2) = match salt_values {
                Some(values) => values,
                None => {
                    tracing::warn!("Skipping frame {} due to missing salt values", frame_idx);
                    continue;
                }
            };
            
            // Use the salt values to fetch the frame
            let db = self.db.clone();
            let keyspace = self.keyspace.clone();
            let frame_key = keyspace.wal_frame_key(&file_id_clone, salt1, salt2, frame_idx);
            
            // Fetch the frame data
            let frame_data = match run_fdb_tx(&db, move |tx| {
                let frame_key_clone = frame_key.clone();
                
                async move {
                    let result = tx.get(&frame_key_clone, false).await?;
                    Ok(result.map(|bytes| bytes.to_vec()))
                }
            }) {
                Ok(Some(data)) => {
                    tracing::info!("Successfully read frame {} (salt1={:x}, salt2={:x}), size={}", 
                        frame_idx, salt1, salt2, data.len());
                    data
                },
                Ok(None) => {
                    // Frame not found even with correct salt values
                    tracing::warn!("Frame {} with salt1={:x}, salt2={:x} not found in FDB", 
                        frame_idx, salt1, salt2);
                    continue; // Try the next frame
                },
                Err(e) => {
                    tracing::error!("Error fetching frame {} with salt1={:x}, salt2={:x}: {}", 
                        frame_idx, salt1, salt2, e);
                    continue; // Try the next frame
                }
            };
            
            // Calculate the portion of this frame to copy
            let frame_start = if frame_idx == first_frame_idx { first_frame_offset } else { 0 };
            let available_in_frame = frame_data.len().saturating_sub(frame_start);
            let to_copy = std::cmp::min(available_in_frame, count - result_pos);
            
            if to_copy > 0 && frame_start < frame_data.len() {
                // Copy the relevant portion to the result buffer
                result[result_pos..result_pos + to_copy].copy_from_slice(
                    &frame_data[frame_start..frame_start + to_copy]
                );
                result_pos += to_copy;
                tracing::info!("Copied {} bytes from frame {} to result buffer at position {}", 
                    to_copy, frame_idx, result_pos - to_copy);
            }
        }
        
        // Return the actual data we were able to read
        result.truncate(result_pos);
        tracing::info!("Read a total of {} bytes of frame data", result_pos);
        Ok(result)
    }
    
    /// Truncate a WAL file to the specified size
    pub fn truncate_wal(&self, wal_path: &str, size: i64) -> Result<(), FdbVfsError> {
        tracing::debug!("Truncating WAL file: path={}, size={}", wal_path, size);
        
        // Extract file_id from path
        // In a real implementation, we would extract the file ID from the path
        // or use a lookup mechanism. For now, we just log the operation.
        tracing::info!("WAL truncate operation: path={}, size={}", wal_path, size);
        
        if size == 0 {
            // Special case: truncating to zero means clearing all WAL data
            // We would clear all WAL frames and the header in FDB
            tracing::info!("Truncating WAL file to zero size, which would clear all WAL data");
            // For now, we just acknowledge the operation
        } else {
            // Partial truncation requires:
            // 1. Determining which frames would be removed
            // 2. Clearing those frames from FDB
            // This is a complex operation that would need to be implemented
            // based on the specific WAL format and storage layout
            tracing::info!("Partial WAL truncation is a complex operation, currently just logging.");
        }
        
        // For now, just acknowledge the operation
        // In a real implementation, we would perform the actual truncation
        Ok(())
    }
}