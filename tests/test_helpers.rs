#![allow(dead_code)]

use std::io::Write;
use tempfile::NamedTempFile;

/// Create a temp file with the given byte content.
pub fn create_temp_file(content: &[u8]) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(content).unwrap();
    f.flush().unwrap();
    f
}

/// Create a temp file with the given string content.
pub fn create_temp_text_file(content: &str) -> NamedTempFile {
    create_temp_file(content.as_bytes())
}
