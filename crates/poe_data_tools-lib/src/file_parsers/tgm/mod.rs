use crate::file_parsers::{FileParser2, error::Result};

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
