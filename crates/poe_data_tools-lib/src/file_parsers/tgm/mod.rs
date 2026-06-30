use crate::file_parsers::{FileParser2, VersionedFile, error::Result};

pub mod parser;
pub mod types;
use parser::parse_tgm_bytes;
use types::TGMFile;

pub struct TGMParser;

impl FileParser2 for TGMParser {
    type Output = TGMFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_tgm_bytes(bytes)
    }
}

impl VersionedFile for TGMFile {
    fn version(&self) -> Option<u32> {
        Some(self.version as u32)
    }
}
