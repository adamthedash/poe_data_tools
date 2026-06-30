use crate::file_parsers::{FileParser2, VersionedFile, error::Result};

pub mod parser;
pub mod types;
use parser::parse_psg_bytes;
use types::PSGFile;

// TODO: Make version private, add POE1, POE2 consts
pub struct PSGParser {
    pub version: u32,
}

impl FileParser2 for PSGParser {
    type Output = PSGFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_psg_bytes(bytes, self.version)
    }
}

impl VersionedFile for PSGFile {
    fn version(&self) -> Option<u32> {
        Some(self.version as u32)
    }
}
