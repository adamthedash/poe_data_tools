use anyhow::Result;

use crate::file_parsers::FileParser;

pub mod parser;
pub mod types;
use parser::parse_amd_str;
use types::AMDFile;

pub struct AMDParser;

impl FileParser for AMDParser {
    type Output = AMDFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = String::from_utf16le(bytes)?;

        parse_amd_str(&contents)
    }
}
