pub mod parser;
pub mod types;
use parser::parse_fmt;
use types::*;

use crate::file_parsers::{FileParser2, VersionedFile, error::Result};

pub struct FMTParser;

impl FileParser2 for FMTParser {
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
