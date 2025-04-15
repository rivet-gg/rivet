use std::fs::File;
use std::io::Read;
use std::path::Path;

use insta::assert_debug_snapshot;
use sqlite_vfs_fdb::wal_parser::{WalFrame, WalIterator, WalParser};

#[test]
fn test_wal_parser_callback() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("tests/fixtures/test_wal");
    let mut file = File::open(path)?;
    
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    let mut parser = WalParser::new();
    let mut frames = Vec::new();
    
    // Add all data at once
    parser.add_data(&data);
    
    // Process and collect frames
    let processed = parser.process(|frame| frames.push(frame))?;
    
    // Take just first 3 frames for the snapshot to keep it manageable
    let preview_frames = if frames.len() > 3 {
        &frames[0..3]
    } else {
        &frames
    };
    
    #[derive(Debug)]
    struct TestResult<'a> {
        processed_count: usize,
        total_frames: usize,
        preview_frames: &'a [WalFrame],
    }
    
    assert_debug_snapshot!("wal_parser_callback", TestResult {
        processed_count: processed,
        total_frames: frames.len(),
        preview_frames,
    });
    
    Ok(())
}

#[test]
fn test_wal_parser_incremental() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("tests/fixtures/test_wal");
    let mut file = File::open(path)?;
    
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    let mut parser = WalParser::new();
    let mut frames = Vec::new();
    let chunk_size = 128; // Small chunk size to test incremental parsing
    
    // Add data in chunks to simulate incremental parsing
    let mut total_processed = 0;
    for chunk in data.chunks(chunk_size) {
        parser.add_data(chunk);
        let processed = parser.process(|frame| frames.push(frame))?;
        total_processed += processed;
    }
    
    // Take just first 3 frames for the snapshot to keep it manageable
    let preview_frames = if frames.len() > 3 {
        &frames[0..3]
    } else {
        &frames
    };
    
    #[derive(Debug)]
    struct TestResult<'a> {
        processed_count: usize,
        total_frames: usize,
        preview_frames: &'a [WalFrame],
    }
    
    assert_debug_snapshot!("wal_parser_incremental", TestResult {
        processed_count: total_processed,
        total_frames: frames.len(),
        preview_frames,
    });
    
    Ok(())
}

#[test]
fn test_wal_iterator() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("tests/fixtures/test_wal");
    let file = File::open(path)?;
    
    let iterator = WalIterator::new(file);
    let frames: Result<Vec<WalFrame>, _> = iterator.collect();
    let frames = frames?;
    
    // Take just first 3 frames for the snapshot to keep it manageable
    let preview_frames = if frames.len() > 3 {
        &frames[0..3]
    } else {
        &frames
    };
    
    #[derive(Debug)]
    struct TestResult<'a> {
        total_frames: usize,
        preview_frames: &'a [WalFrame],
    }
    
    assert_debug_snapshot!("wal_iterator", TestResult {
        total_frames: frames.len(),
        preview_frames,
    });
    
    Ok(())
}

#[test]
fn test_compare_parser_methods() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("tests/fixtures/test_wal");
    let mut file = File::open(path)?;
    
    // Read all data for callback approach
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    // Collect frames using callback approach
    let mut parser = WalParser::new();
    let mut callback_frames = Vec::new();
    parser.add_data(&data);
    parser.process(|frame| callback_frames.push(frame))?;
    
    // Collect frames using iterator approach
    let file = File::open(path)?;
    let iterator = WalIterator::new(file);
    let iterator_frames: Result<Vec<WalFrame>, _> = iterator.collect();
    let iterator_frames = iterator_frames?;
    
    // Compare counts for snapshot
    #[derive(Debug)]
    struct TestResult<'a> {
        callback_frame_count: usize,
        iterator_frame_count: usize,
        counts_match: bool,
        first_callback_frame: Option<&'a WalFrame>,
        first_iterator_frame: Option<&'a WalFrame>,
    }
    
    assert_debug_snapshot!("compare_parser_methods", TestResult {
        callback_frame_count: callback_frames.len(),
        iterator_frame_count: iterator_frames.len(),
        counts_match: callback_frames.len() == iterator_frames.len(),
        first_callback_frame: if !callback_frames.is_empty() { Some(&callback_frames[0]) } else { None },
        first_iterator_frame: if !iterator_frames.is_empty() { Some(&iterator_frames[0]) } else { None },
    });
    
    // Also perform assertions to ensure test fails if something is wrong
    assert_eq!(
        callback_frames.len(),
        iterator_frames.len(),
        "Both methods should parse the same number of frames"
    );
    
    if !callback_frames.is_empty() && !iterator_frames.is_empty() {
        assert_eq!(
            callback_frames[0].page_number, 
            iterator_frames[0].page_number,
            "First frames should have the same page number"
        );
    }
    
    Ok(())
}