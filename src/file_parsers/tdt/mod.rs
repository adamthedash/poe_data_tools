pub mod parser;
pub mod types;
use anyhow::{Result, anyhow};
use parser::parse_tdt_bytes;
use types::*;

use crate::file_parsers::FileParser;

pub struct TDTParser;

impl FileParser for TDTParser {
    type Output = TDTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_tdt_bytes(bytes).map_err(|e| anyhow!("Failed to parse file: {:?}", e))
    }
}
