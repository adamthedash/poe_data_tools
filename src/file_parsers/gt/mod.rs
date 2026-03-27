
use crate::file_parsers::{FileParser, VersionedResult, shared::utf16_bom_to_string};

pub mod parser;
pub mod types;
use parser::parse_gt_str;
use types::*;

pub struct GTParser;

impl FileParser for GTParser {
    type Output = GTFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        Ok(parse_gt_str(&contents)?)
    }
}
