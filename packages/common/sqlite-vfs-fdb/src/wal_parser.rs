use std::io::{self, Read};

#[cfg(test)]
use serde::Serialize;
use std::fmt::Debug;

/// WAL file magic number: SQLite format 3
/// The spec states "Write Ahead Log\0" but in reality it's different
const WAL_MAGIC: [u8; 4] = [0x37, 0x7F, 0x06, 0x82];

/// Size of the WAL header
const WAL_HEADER_SIZE: usize = 32;

/// Size of the WAL frame header
const FRAME_HEADER_SIZE: usize = 24;

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
                    
                    let page_number = u32::from_be_bytes([
                        self.buffer[self.position],
                        self.buffer[self.position + 1],
                        self.buffer[self.position + 2],
                        self.buffer[self.position + 3],
                    ]);

                    let database_size = u32::from_be_bytes([
                        self.buffer[self.position + 4],
                        self.buffer[self.position + 5],
                        self.buffer[self.position + 6],
                        self.buffer[self.position + 7],
                    ]);

                    let salt_1 = u32::from_be_bytes([
                        self.buffer[self.position + 8],
                        self.buffer[self.position + 9],
                        self.buffer[self.position + 10],
                        self.buffer[self.position + 11],
                    ]);

                    let salt_2 = u32::from_be_bytes([
                        self.buffer[self.position + 12],
                        self.buffer[self.position + 13],
                        self.buffer[self.position + 14],
                        self.buffer[self.position + 15],
                    ]);

                    let checksum_1 = u32::from_be_bytes([
                        self.buffer[self.position + 16],
                        self.buffer[self.position + 17],
                        self.buffer[self.position + 18],
                        self.buffer[self.position + 19],
                    ]);

                    let checksum_2 = u32::from_be_bytes([
                        self.buffer[self.position + 20],
                        self.buffer[self.position + 21],
                        self.buffer[self.position + 22],
                        self.buffer[self.position + 23],
                    ]);

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
        // Check magic number (first 4 bytes)
        let magic_slice = &self.buffer[self.position..self.position + 4];
        if magic_slice != WAL_MAGIC {
            return Err(WalParseError::InvalidMagic);
        }

        // Skip ahead to format version (starts at byte 4)
        let format_version = u32::from_be_bytes([
            self.buffer[self.position + 4],
            self.buffer[self.position + 5],
            self.buffer[self.position + 6],
            self.buffer[self.position + 7],
        ]);

        let page_size = u32::from_be_bytes([
            self.buffer[self.position + 8],
            self.buffer[self.position + 9],
            self.buffer[self.position + 10],
            self.buffer[self.position + 11],
        ]);

        // Page size must be a power of 2 between 512 and 32768
        if page_size < 512 || page_size > 32768 || (page_size & (page_size - 1)) != 0 {
            return Err(WalParseError::InvalidPageSize(page_size));
        }

        let checkpoint_sequence = u32::from_be_bytes([
            self.buffer[self.position + 12],
            self.buffer[self.position + 13],
            self.buffer[self.position + 14],
            self.buffer[self.position + 15],
        ]);

        let salt_1 = u32::from_be_bytes([
            self.buffer[self.position + 16],
            self.buffer[self.position + 17],
            self.buffer[self.position + 18],
            self.buffer[self.position + 19],
        ]);

        let salt_2 = u32::from_be_bytes([
            self.buffer[self.position + 20],
            self.buffer[self.position + 21],
            self.buffer[self.position + 22],
            self.buffer[self.position + 23],
        ]);

        let checksum_1 = u32::from_be_bytes([
            self.buffer[self.position + 24],
            self.buffer[self.position + 25],
            self.buffer[self.position + 26],
            self.buffer[self.position + 27],
        ]);

        let checksum_2 = u32::from_be_bytes([
            self.buffer[self.position + 28],
            self.buffer[self.position + 29],
            self.buffer[self.position + 30],
            self.buffer[self.position + 31],
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