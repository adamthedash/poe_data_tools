use crate::file_parsers::{FileParser, VersionedResult};

pub mod parser;
pub mod types;
use parser::parse_psg_bytes;
use types::PSGFile;

pub struct PSGParser {
    pub version: u32,
}

impl FileParser for PSGParser {
    type Output = PSGFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        parse_psg_bytes(bytes, self.version)
    }
}
