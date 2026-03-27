use anyhow::Context;

use crate::file_parsers::{FileParser, VersionedResult};

pub mod parser;
pub mod types;
use parser::parse_amd_str;
use types::AMDFile;

pub struct AMDParser;

impl FileParser for AMDParser {
    type Output = AMDFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        let contents = String::from_utf16le(bytes).context("Failed to parse bytes as utf16LE")?;

        parse_amd_str(&contents)
    }
}
