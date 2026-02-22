pub mod nom_parser;
pub mod types;
pub mod winnow_parser;

use anyhow::Result;
use types::*;
use winnow_parser::parse_rs_str;

use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub struct RSParser;

impl FileParser for RSParser {
    type Output = RSFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_rs_str(&contents)
    }
}
