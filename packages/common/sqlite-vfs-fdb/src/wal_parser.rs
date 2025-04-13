use std::io::{self, Read};
use std::fmt::Debug;

#[cfg(test)]
use serde::Serialize;

/// WAL file magic number
const WAL_MAGIC: [u8; 4] = [0x37, 0x7F, 0x06, 0x82];
/// Full WAL magic number with version
const WAL_HEADER_MAGIC: [u8; 8] = [0x37, 0x7F, 0x06, 0x82, 0x00, 0xD9, 0x2C, 0x00];

/// Size of the WAL header
pub const WAL_HEADER_SIZE: usize = 32;

/// Size of the WAL frame header
pub const FRAME_HEADER_SIZE: usize = 24;

/// Represents a WAL file header
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Serialize))]
pub struct WalHeader {
    pub format_version: u32,
    pub page_size: u32,
    pub checkpoint_sequence: u32,
    pub salt_1: u32,
    pub salt_2: u32,
    pub checksum_1: u32,
    pub checksum_2: u32,
}

impl WalHeader {
    /// Creates a new WAL header with default values
    pub fn new(page_size: u32) -> Self {
        Self {
            format_version: 3007000, // Current SQLite WAL format version
            page_size,
            checkpoint_sequence: 0,
            salt_1: rand::random::<u32>(),
            salt_2: rand::random::<u32>(),
            checksum_1: 0,
            checksum_2: 0,
        }
    }

    /// Serializes the WAL header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(WAL_HEADER_SIZE);
        
        // Magic number and format version (8 bytes)
        buffer.extend_from_slice(&WAL_HEADER_MAGIC);
        
        // Page size (4 bytes)
        buffer.extend_from_slice(&self.page_size.to_be_bytes());
        
        // Checkpoint sequence (4 bytes)
        buffer.extend_from_slice(&self.checkpoint_sequence.to_be_bytes());
        
        // Salt values (8 bytes)
        buffer.extend_from_slice(&self.salt_1.to_be_bytes());
        buffer.extend_from_slice(&self.salt_2.to_be_bytes());
        
        // Checksum values (8 bytes)
        buffer.extend_from_slice(&self.checksum_1.to_be_bytes());
        buffer.extend_from_slice(&self.checksum_2.to_be_bytes());
        
        buffer
    }

    /// Deserializes a WAL header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WalParseError> {
        if bytes.len() < WAL_HEADER_SIZE {
            return Err(WalParseError::IncompleteData);
        }

        // Check magic number
        if bytes[0..4] != WAL_MAGIC {
            return Err(WalParseError::InvalidMagic);
        }

        let format_version = u32::from_be_bytes([
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);

        let page_size = u32::from_be_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11],
        ]);

        // Page size must be a power of 2 between 512 and 32768
        if page_size < 512 || page_size > 32768 || (page_size & (page_size - 1)) != 0 {
            return Err(WalParseError::InvalidPageSize(page_size));
        }

        let checkpoint_sequence = u32::from_be_bytes([
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]);

        let salt_1 = u32::from_be_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19],
        ]);

        let salt_2 = u32::from_be_bytes([
            bytes[20], bytes[21], bytes[22], bytes[23],
        ]);

        let checksum_1 = u32::from_be_bytes([
            bytes[24], bytes[25], bytes[26], bytes[27],
        ]);

        let checksum_2 = u32::from_be_bytes([
            bytes[28], bytes[29], bytes[30], bytes[31],
        ]);

        Ok(WalHeader {
            format_version,
            page_size,
            checkpoint_sequence,
            salt_1,
            salt_2,
            checksum_1,
            checksum_2,
        })
    }
}

/// Represents a WAL frame
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Serialize))]
pub struct WalFrame {
    pub page_number: u32,
    pub database_size: u32,
    pub salt_1: u32,
    pub salt_2: u32,
    pub checksum_1: u32,
    pub checksum_2: u32,
    #[cfg_attr(test, serde(skip))]
    pub page_data: Vec<u8>,
    #[cfg(test)]
    pub page_data_len: usize,
    #[cfg(test)]
    pub page_data_preview: [u8; 16],
}

impl WalFrame {
    /// Creates a new WAL frame
    pub fn new(page_number: u32, database_size: u32, salt_1: u32, salt_2: u32, page_data: Vec<u8>) -> Self {
        Self {
            page_number,
            database_size,
            salt_1,
            salt_2,
            checksum_1: 0, // We'll calculate this when needed
            checksum_2: 0, // We'll calculate this when needed
            #[cfg(test)]
            page_data_len: page_data.len(),
            #[cfg(test)]
            page_data_preview: {
                let mut preview = [0u8; 16];
                let preview_len = std::cmp::min(16, page_data.len());
                preview[..preview_len].copy_from_slice(&page_data[..preview_len]);
                preview
            },
            page_data,
        }
    }

    /// Serializes the frame header to bytes (not including page data)
    pub fn header_to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(FRAME_HEADER_SIZE);
        
        // Page number (4 bytes)
        buffer.extend_from_slice(&self.page_number.to_be_bytes());
        
        // Database size in pages (4 bytes)
        buffer.extend_from_slice(&self.database_size.to_be_bytes());
        
        // Salt values (8 bytes)
        buffer.extend_from_slice(&self.salt_1.to_be_bytes());
        buffer.extend_from_slice(&self.salt_2.to_be_bytes());
        
        // Checksum values (8 bytes)
        buffer.extend_from_slice(&self.checksum_1.to_be_bytes());
        buffer.extend_from_slice(&self.checksum_2.to_be_bytes());
        
        buffer
    }

    /// Deserializes a frame header from bytes
    pub fn header_from_bytes(bytes: &[u8]) -> Result<(u32, u32, u32, u32, u32, u32), WalParseError> {
        if bytes.len() < FRAME_HEADER_SIZE {
            return Err(WalParseError::IncompleteData);
        }

        let page_number = u32::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
        ]);

        let database_size = u32::from_be_bytes([
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);

        let salt_1 = u32::from_be_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11],
        ]);

        let salt_2 = u32::from_be_bytes([
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]);

        let checksum_1 = u32::from_be_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19],
        ]);

        let checksum_2 = u32::from_be_bytes([
            bytes[20], bytes[21], bytes[22], bytes[23],
        ]);

        Ok((page_number, database_size, salt_1, salt_2, checksum_1, checksum_2))
    }
}

/// Parser state for processing WAL data
#[derive(Debug, PartialEq)]
enum ParserState {
    ReadingHeader,
    ReadingFrameHeader,
    ReadingFrameData {
        page_number: u32,
        database_size: u32,
        salt_1: u32,
        salt_2: u32,
        checksum_1: u32,
        checksum_2: u32,
        page_size: usize,
    },
}

/// Errors that can occur during WAL parsing
#[derive(Debug, thiserror::Error)]
pub enum WalParseError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Invalid WAL magic number")]
    InvalidMagic,

    #[error("Not enough data to parse header or frame")]
    IncompleteData,

    #[error("Invalid WAL format version")]
    InvalidFormatVersion,

    #[error("Invalid page size: {0}")]
    InvalidPageSize(u32),

    #[error("Invalid checksum")]
    InvalidChecksum,
    
    #[error("No main database found for WAL file")]
    NoMainDatabase,
}

/// WAL parser that processes data incrementally
pub struct WalParser {
    buffer: Vec<u8>,
    position: usize,
    state: ParserState,
    header: Option<WalHeader>,
}

impl WalParser {
    /// Create a new WAL parser
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
            position: 0,
            state: ParserState::ReadingHeader,
            header: None,
        }
    }

    /// Add data to the parser buffer
    pub fn add_data(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Process available data and call the callback for each complete frame
    pub fn process<F>(&mut self, mut callback: F) -> Result<usize, WalParseError>
    where
        F: FnMut(WalFrame),
    {
        let mut processed = 0;

        loop {
            let available = self.buffer.len() - self.position;
            
            match self.state {
                ParserState::ReadingHeader => {
                    if available < WAL_HEADER_SIZE {
                        break; // Need more data
                    }

                    let header = self.parse_header()?;
                    self.header = Some(header);
                    self.position += WAL_HEADER_SIZE;
                    self.state = ParserState::ReadingFrameHeader;
                }

                ParserState::ReadingFrameHeader => {
                    if available < FRAME_HEADER_SIZE {
                        break; // Need more data
                    }

                    let header = self.header.as_ref().unwrap();
                    let page_size = header.page_size as usize;
                    
                    let (page_number, database_size, salt_1, salt_2, checksum_1, checksum_2) = 
                        WalFrame::header_from_bytes(&self.buffer[self.position..self.position + FRAME_HEADER_SIZE])?;

                    self.position += FRAME_HEADER_SIZE;
                    self.state = ParserState::ReadingFrameData {
                        page_number,
                        database_size,
                        salt_1,
                        salt_2,
                        checksum_1,
                        checksum_2,
                        page_size,
                    };
                }

                ParserState::ReadingFrameData {
                    page_number,
                    database_size,
                    salt_1,
                    salt_2,
                    checksum_1,
                    checksum_2,
                    page_size,
                } => {
                    if available < page_size {
                        break; // Need more data
                    }

                    let frame_data = self.buffer[self.position..self.position + page_size].to_vec();
                    self.position += page_size;

                    // We're not validating checksums in this implementation
                    #[cfg(test)]
                    let frame = {
                        // Create a 16-byte preview of page data
                        let mut preview = [0u8; 16];
                        let preview_len = std::cmp::min(16, frame_data.len());
                        preview[..preview_len].copy_from_slice(&frame_data[..preview_len]);
                        
                        WalFrame {
                            page_number,
                            database_size,
                            salt_1,
                            salt_2,
                            checksum_1,
                            checksum_2,
                            page_data: frame_data.clone(),
                            page_data_len: frame_data.len(),
                            page_data_preview: preview,
                        }
                    };
                    
                    #[cfg(not(test))]
                    let frame = WalFrame {
                        page_number,
                        database_size,
                        salt_1,
                        salt_2,
                        checksum_1,
                        checksum_2,
                        page_data: frame_data,
                    };

                    callback(frame);
                    processed += 1;
                    self.state = ParserState::ReadingFrameHeader;
                }
            }
        }

        // Compact buffer by removing processed data
        if self.position > 0 {
            self.buffer.drain(0..self.position);
            self.position = 0;
        }

        Ok(processed)
    }

    /// Parse the WAL header
    fn parse_header(&self) -> Result<WalHeader, WalParseError> {
        WalHeader::from_bytes(&self.buffer[self.position..self.position + WAL_HEADER_SIZE])
    }
}

/// A wrapper around WalParser that provides an iterator interface
pub struct WalIterator<R: Read> {
    reader: R,
    parser: WalParser,
    buffer: [u8; 4096],
    frames: Vec<WalFrame>,
    eof_reached: bool,
}

impl<R: Read> WalIterator<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            parser: WalParser::new(),
            buffer: [0; 4096],
            frames: Vec::new(),
            eof_reached: false,
        }
    }

    fn read_more_data(&mut self) -> Result<bool, WalParseError> {
        if self.eof_reached {
            return Ok(false);
        }

        match self.reader.read(&mut self.buffer) {
            Ok(0) => {
                self.eof_reached = true;
                Ok(false) // EOF reached
            }
            Ok(n) => {
                self.parser.add_data(&self.buffer[..n]);
                self.parser.process(|frame| self.frames.push(frame))?;
                Ok(true) // Read some data
            }
            Err(e) => Err(WalParseError::IoError(e)),
        }
    }
}

impl<R: Read> Iterator for WalIterator<R> {
    type Item = Result<WalFrame, WalParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Keep reading data until we have at least one frame or EOF
        while self.frames.is_empty() && !self.eof_reached {
            match self.read_more_data() {
                Ok(false) => {
                    if self.frames.is_empty() {
                        return None; // EOF and no frames left
                    }
                    break;
                }
                Ok(true) => {
                    // Continue reading if no frames were parsed
                    if self.frames.is_empty() {
                        continue;
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }

        // Return the first frame if we parsed any
        if !self.frames.is_empty() {
            return Some(Ok(self.frames.remove(0)));
        }

        // No frames and EOF reached
        None
    }
}