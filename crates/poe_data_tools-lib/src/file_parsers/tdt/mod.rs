pub mod parser;
pub mod types;
use parser::parse_tdt_bytes;
use types::*;

use crate::file_parsers::{FileParser, VersionedFile, error::Result};

pub struct TDTParser;

impl FileParser for TDTParser {
    type Output = TDTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_tdt_bytes(bytes)
    }
}

impl VersionedFile for TDTFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
