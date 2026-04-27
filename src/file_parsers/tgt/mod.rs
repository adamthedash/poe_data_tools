use crate::file_parsers::{FileParser, VersionedResult};

pub mod parser;
pub mod types;
use anyhow::Context;
use parser::parse_tgt_str;
use types::*;

pub struct TGTParser;

impl FileParser for TGTParser {
    type Output = TGTFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        let contents = String::from_utf16le(bytes).context("Failed to parse file as UTF16LE")?;

        parse_tgt_str(&contents)
    }
}
