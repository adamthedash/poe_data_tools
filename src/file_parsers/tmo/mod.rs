use anyhow::Result;

use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub mod parser;
pub mod types;
use parser::parse_tmo_str;
use types::*;

pub struct TMOParser;

impl FileParser for TMOParser {
    type Output = TMOFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_tmo_str(&contents)
    }
}
