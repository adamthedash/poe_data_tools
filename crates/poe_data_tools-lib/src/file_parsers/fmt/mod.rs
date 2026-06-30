pub mod parser;
pub mod types;
use parser::parse_fmt;
use types::*;

use crate::file_parsers::{FileParser, VersionedFile, error::Result};

pub struct FMTParser;

impl FileParser for FMTParser {
    type Output = FMTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_fmt(bytes)
    }
}

impl VersionedFile for FMTFile {
    fn version(&self) -> Option<u32> {
        Some(self.version as u32)
    }
}
