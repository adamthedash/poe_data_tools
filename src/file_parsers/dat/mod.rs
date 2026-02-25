use anyhow::Result;

use crate::file_parsers::FileParser;

pub mod parser;
pub mod types;
use parser::parse_dat_bytes;
use types::DatFile;

pub struct DatParser;

impl FileParser for DatParser {
    type Output = DatFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_dat_bytes(bytes)
    }
}
