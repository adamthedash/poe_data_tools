use crate::file_parsers::{FileParser, VersionedResult};

pub mod parser;
pub mod types;
use anyhow::Context;
use parser::parse_sm_str;
use types::*;

pub struct SMParser;

impl FileParser for SMParser {
    type Output = SMFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        let contents = String::from_utf16le(bytes).context("Failed to parse file as UTF16LE")?;

        parse_sm_str(&contents)
    }
}
