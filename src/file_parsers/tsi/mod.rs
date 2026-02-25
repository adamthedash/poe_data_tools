use anyhow::Result;

use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub mod parser;
pub mod types;

use parser::parse_tsi_str;
pub use types::*;

pub struct TSIParser;

impl FileParser for TSIParser {
    type Output = TSIFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_tsi_str(&contents)
    }
}
