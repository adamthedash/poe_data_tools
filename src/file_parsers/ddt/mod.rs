pub mod nom_parser;
pub mod types;
pub mod winnow_parser;

use anyhow::Result;
use types::*;
use winnow_parser::parse_ddt_str;

use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub struct DDTParser;

impl FileParser for DDTParser {
    type Output = DDTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_ddt_str(&contents)
    }
}
