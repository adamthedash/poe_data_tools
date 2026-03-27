use anyhow::Context;

use crate::file_parsers::{FileParser, VersionedResult, shared::utf16_bom_to_string};

pub mod parser;
pub mod types;
use parser::parse_tst_str;
use types::TSTFile;

pub struct TSTParser;

impl FileParser for TSTParser {
    type Output = TSTFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        let contents = utf16_bom_to_string(bytes)
            .or_else(|_| String::from_utf16le(bytes).context("Failed to parse as UTF16-LE"))?;

        Ok(parse_tst_str(&contents)?)
    }
}
