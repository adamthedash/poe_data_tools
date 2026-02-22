use anyhow::Result;

use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub mod parser;
pub mod types;
use parser::parse_clt_str;
use types::CLTFile;

pub struct CLTParser;

impl FileParser for CLTParser {
    type Output = CLTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_clt_str(&contents)
    }
}
