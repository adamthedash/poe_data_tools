pub mod parser;
pub mod types;
use anyhow::{Context, Result};
use parser::parse_mat_str;
use types::MATFile;

use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub struct MATParser;

impl FileParser for MATParser {
    type Output = MATFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)
            .or_else(|_| String::from_utf16le(bytes).context("Failed to parse as UTF16-LE"))?;

        parse_mat_str(&contents)
    }
}
