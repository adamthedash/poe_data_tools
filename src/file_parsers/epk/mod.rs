use anyhow::{Context, Result};

use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub mod parser;
pub mod types;
use parser::parse_epk_str;
use types::EPKFile;

pub struct EPKParser;

impl FileParser for EPKParser {
    type Output = EPKFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)
            .or_else(|_| String::from_utf8(bytes.to_vec()).context("Failed to parse as UTF-8"))?;

        parse_epk_str(&contents)
    }
}
