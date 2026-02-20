use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub mod parser;
pub mod types;
pub mod winnow_parser;
use types::*;
use winnow_parser::parse_gft_str;

pub struct GFTParser;

impl FileParser for GFTParser {
    type Output = GFTFile;

    fn parse(&self, bytes: &[u8]) -> anyhow::Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_gft_str(&contents)
    }
}
