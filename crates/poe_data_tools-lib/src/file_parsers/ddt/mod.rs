pub mod parser;
pub mod types;

use parser::parse_ddt_str;
use types::*;

use crate::file_parsers::{FileParser, VersionedResult, shared::utf16_bom_to_string};

pub struct DDTParser;

impl FileParser for DDTParser {
    type Output = DDTFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_ddt_str(&contents)
    }
}
