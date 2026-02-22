use anyhow::Result;

use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub mod nom_parser;
pub mod types;
pub mod winnow_parser;
use types::*;
use winnow_parser::parse_ecf_str;

pub struct ECFParser;

impl FileParser for ECFParser {
    type Output = EcfFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_ecf_str(&contents)
    }
}
