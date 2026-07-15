pub mod parser;
pub mod types;

use parser::parse_smd;
use types::SMDFile;

use crate::file_parsers::{FileParser, VersionedFile, error::Result};

pub struct SMDParser;

impl FileParser for SMDParser {
    type Output = SMDFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_smd(bytes)
    }
}

impl VersionedFile for SMDFile {
    fn version(&self) -> Option<u32> {
        Some(self.version as u32)
    }
}
