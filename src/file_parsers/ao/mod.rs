pub mod parser;
pub mod types;

use anyhow::{Result, anyhow};
use parser::parse_ao_str;
use types::*;

use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub struct AOParser;

impl FileParser for AOParser {
    type Output = AOFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let (_, parsed) =
            parse_ao_str(contents.trim()).map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

        Ok(parsed)
    }
}
