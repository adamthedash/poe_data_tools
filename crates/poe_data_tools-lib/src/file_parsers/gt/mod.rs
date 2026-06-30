use crate::file_parsers::{
    FileParser, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_gt_str;
use types::*;

pub struct GTParser;

impl FileParser for GTParser {
    type Output = GTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_gt_str(&contents)
    }
}

impl VersionedFile for GTFile {
    fn version(&self) -> Option<u32> {
        None
    }
}
