use crate::file_parsers::{FileParser, VersionedResult};

pub mod parser;
pub mod types;
use parser::parse_tgm_bytes;
use types::TGMFile;

pub struct TGMParser;

impl FileParser for TGMParser {
    type Output = TGMFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        parse_tgm_bytes(bytes)
    }
}
