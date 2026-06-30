use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_dct_str;
use types::*;

pub struct DCTParser;

impl FileParser2 for DCTParser {
    type Output = DCTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_dct_str(&contents)
    }
}
