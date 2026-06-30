use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_tmo_str;
use types::*;

pub struct TMOParser;

impl FileParser2 for TMOParser {
    type Output = TMOFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_tmo_str(&contents)
    }
}
