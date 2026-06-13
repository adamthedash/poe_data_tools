pub mod parser;
pub mod types;

use parser::parse_rs_str;
use types::*;

use crate::file_parsers::{FileParser, VersionedResult, shared::utf16_bom_to_string};

pub struct RSParser;

impl FileParser for RSParser {
    type Output = RSFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_rs_str(&contents)
    }
}
