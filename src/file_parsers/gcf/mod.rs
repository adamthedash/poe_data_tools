use anyhow::Result;

use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub mod parser;
pub mod types;
use parser::parse_gcf_str;
use types::*;

pub struct GCFParser;

impl FileParser for GCFParser {
    type Output = GcfFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_gcf_str(&contents)
    }
}
