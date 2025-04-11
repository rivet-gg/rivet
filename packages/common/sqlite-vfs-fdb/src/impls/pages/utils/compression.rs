use bytes::{Bytes, BytesMut};
use std::fmt;
use thiserror::Error;

use crate::metrics;

/// Compression error types
#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("LZ4 compression error: {0}")]
    Lz4Error(String),

    #[error("Snappy compression error: {0}")]
    SnappyError(String),

    #[error("Zstd compression error: {0}")]
    ZstdError(String),

    #[error("Unknown compression type: {0}")]
    UnknownType(u8),
}

/// Available compression types for SQLite VFS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionType {
    /// No compression
    None = 0,
    
    /// LZ4 compression (fast)
    Lz4 = 1,
    
    /// Snappy compression (fast)
    Snappy = 2,
    
    /// Zstandard compression (better ratio)
    Zstd = 3,
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::None
    }
}

impl From<u8> for CompressionType {
    fn from(value: u8) -> Self {
        match value {
            0 => CompressionType::None,
            1 => CompressionType::Lz4,
            2 => CompressionType::Snappy,
            3 => CompressionType::Zstd,
            _ => CompressionType::None,
        }
    }
}

impl From<CompressionType> for u8 {
    fn from(value: CompressionType) -> Self {
        match value {
            CompressionType::None => 0,
            CompressionType::Lz4 => 1,
            CompressionType::Snappy => 2,
            CompressionType::Zstd => 3,
        }
    }
}

impl fmt::Display for CompressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionType::None => write!(f, "none"),
            CompressionType::Lz4 => write!(f, "lz4"),
            CompressionType::Snappy => write!(f, "snappy"),
            CompressionType::Zstd => write!(f, "zstd"),
        }
    }
}

/// Trait for compression implementations
pub trait Compression: Send + Sync {
    /// Compress data
    fn compress(&self, data: &[u8]) -> Result<Bytes, CompressionError>;
    
    /// Decompress data
    fn decompress(&self, compressed_data: &[u8], expected_size: usize) -> Result<Bytes, CompressionError>;
    
    /// Get the compression type
    fn compression_type(&self) -> CompressionType;
    
    /// Get the VFS name suffix
    fn vfs_name_suffix(&self) -> &'static str;
    
    /// Create a new instance of the same compressor
    fn box_clone(&self) -> Box<dyn Compression>;
}

/// No compression implementation
pub struct NoCompression;

impl Compression for NoCompression {
    fn compress(&self, data: &[u8]) -> Result<Bytes, CompressionError> {
        Ok(Bytes::copy_from_slice(data))
    }
    
    fn decompress(&self, compressed_data: &[u8], _expected_size: usize) -> Result<Bytes, CompressionError> {
        Ok(Bytes::copy_from_slice(compressed_data))
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::None
    }
    
    fn vfs_name_suffix(&self) -> &'static str {
        ""
    }
    
    fn box_clone(&self) -> Box<dyn Compression> {
        Box::new(NoCompression)
    }
}

/// LZ4 compression implementation
pub struct Lz4Compression {
    /// Compression level (1-12)
    level: u32,
}

impl Lz4Compression {
    pub fn new(level: u32) -> Self {
        // Ensure level is within valid range
        let level = level.clamp(1, 12);
        Self { level }
    }
}

impl Default for Lz4Compression {
    fn default() -> Self {
        Self::new(6) // Default to medium compression level
    }
}

impl Compression for Lz4Compression {
    fn compress(&self, data: &[u8]) -> Result<Bytes, CompressionError> {
        let timer = metrics::start_compression_operation();
        
        // Compress the data using LZ4
        let mut mode = lz4::block::CompressionMode::FAST(self.level as i32);
        if self.level > 6 {
            mode = lz4::block::CompressionMode::HIGHCOMPRESSION(self.level as i32);
        }
        
        let compressed = lz4::block::compress(data, Some(mode), false)
            .map_err(|e| CompressionError::Lz4Error(e.to_string()))?;
        
        // Store the original size at the beginning for decompression
        let mut result = BytesMut::with_capacity(4 + compressed.len());
        result.extend_from_slice(&(data.len() as u32).to_le_bytes());
        result.extend_from_slice(&compressed);
        
        // Record metrics
        metrics::complete_compression_operation(
            &timer, 
            CompressionType::Lz4, 
            data.len(), 
            result.len(),
            true
        );
        
        Ok(result.freeze())
    }
    
    fn decompress(&self, compressed_data: &[u8], _expected_size: usize) -> Result<Bytes, CompressionError> {
        let timer = metrics::start_compression_operation();
        
        if compressed_data.len() < 4 {
            return Err(CompressionError::Lz4Error("Compressed data too short".to_string()));
        }
        
        // Extract the original size
        let mut size_bytes = [0u8; 4];
        size_bytes.copy_from_slice(&compressed_data[0..4]);
        let _original_size = u32::from_le_bytes(size_bytes) as usize;
        
        // Decompress the data
        let decompressed = lz4::block::decompress(&compressed_data[4..], None)
            .map_err(|e| CompressionError::Lz4Error(e.to_string()))?;
        
        // Record metrics
        metrics::complete_compression_operation(
            &timer, 
            CompressionType::Lz4, 
            decompressed.len(), 
            compressed_data.len(),
            false
        );
        
        Ok(Bytes::from(decompressed))
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::Lz4
    }
    
    fn vfs_name_suffix(&self) -> &'static str {
        "_lz4"
    }
    
    fn box_clone(&self) -> Box<dyn Compression> {
        Box::new(Lz4Compression { level: self.level })
    }
}

/// Snappy compression implementation
pub struct SnappyCompression;

impl Compression for SnappyCompression {
    fn compress(&self, data: &[u8]) -> Result<Bytes, CompressionError> {
        let timer = metrics::start_compression_operation();
        
        // Compress the data
        let compressed = snap::raw::Encoder::new()
            .compress_vec(data)
            .map_err(|e| CompressionError::SnappyError(e.to_string()))?;
        
        // Record metrics
        metrics::complete_compression_operation(
            &timer, 
            CompressionType::Snappy, 
            data.len(), 
            compressed.len(),
            true
        );
        
        Ok(Bytes::from(compressed))
    }
    
    fn decompress(&self, compressed_data: &[u8], _expected_size: usize) -> Result<Bytes, CompressionError> {
        let timer = metrics::start_compression_operation();
        
        // Decompress the data
        let decompressed = snap::raw::Decoder::new()
            .decompress_vec(compressed_data)
            .map_err(|e| CompressionError::SnappyError(e.to_string()))?;
        
        // Record metrics
        metrics::complete_compression_operation(
            &timer, 
            CompressionType::Snappy, 
            decompressed.len(), 
            compressed_data.len(),
            false
        );
        
        Ok(Bytes::from(decompressed))
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::Snappy
    }
    
    fn vfs_name_suffix(&self) -> &'static str {
        "_snappy"
    }
    
    fn box_clone(&self) -> Box<dyn Compression> {
        Box::new(SnappyCompression)
    }
}

/// Zstd compression implementation
pub struct ZstdCompression {
    /// Compression level (1-22)
    level: i32,
}

impl ZstdCompression {
    pub fn new(level: i32) -> Self {
        // Ensure level is within valid range
        let level = level.clamp(1, 22);
        Self { level }
    }
}

impl Default for ZstdCompression {
    fn default() -> Self {
        Self::new(3) // Default to medium compression level
    }
}

impl Compression for ZstdCompression {
    fn compress(&self, data: &[u8]) -> Result<Bytes, CompressionError> {
        let timer = metrics::start_compression_operation();
        
        // Compress the data
        let compressed = zstd::encode_all(data, self.level)
            .map_err(|e| CompressionError::ZstdError(e.to_string()))?;
        
        // Record metrics
        metrics::complete_compression_operation(
            &timer, 
            CompressionType::Zstd, 
            data.len(), 
            compressed.len(),
            true
        );
        
        Ok(Bytes::from(compressed))
    }
    
    fn decompress(&self, compressed_data: &[u8], _expected_size: usize) -> Result<Bytes, CompressionError> {
        let timer = metrics::start_compression_operation();
        
        // Decompress the data
        let decompressed = zstd::decode_all(compressed_data)
            .map_err(|e| CompressionError::ZstdError(e.to_string()))?;
        
        // Record metrics
        metrics::complete_compression_operation(
            &timer, 
            CompressionType::Zstd, 
            decompressed.len(), 
            compressed_data.len(),
            false
        );
        
        Ok(Bytes::from(decompressed))
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::Zstd
    }
    
    fn vfs_name_suffix(&self) -> &'static str {
        "_zstd"
    }
    
    fn box_clone(&self) -> Box<dyn Compression> {
        Box::new(ZstdCompression { level: self.level })
    }
}

/// Factory to create a compressor based on type
pub fn create_compressor(compression_type: CompressionType) -> Box<dyn Compression> {
    match compression_type {
        CompressionType::None => Box::new(NoCompression),
        CompressionType::Lz4 => Box::new(Lz4Compression::default()),
        CompressionType::Snappy => Box::new(SnappyCompression),
        CompressionType::Zstd => Box::new(ZstdCompression::default()),
    }
}