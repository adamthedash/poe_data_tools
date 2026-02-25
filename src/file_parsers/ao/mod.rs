pub mod parser;
pub mod types;

use anyhow::Result;
use parser::parse_ao_str;
use types::*;

use crate::file_parsers::{shared::utf16_bom_to_string, FileParser};

pub struct AOParser;

impl FileParser for AOParser {
    type Output = AOFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_ao_str(contents.trim())
    }
}
