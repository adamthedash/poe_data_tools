use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub mod parser;
pub mod types;
use parser::parse_gft_str;
use types::*;

pub struct GFTParser;

impl FileParser for GFTParser {
    type Output = GFTFile;

    fn parse(&self, bytes: &[u8]) -> anyhow::Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_gft_str(&contents)
    }
}
