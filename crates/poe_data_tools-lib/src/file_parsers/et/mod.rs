use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_et_str;
use types::*;

pub struct ETParser;

impl FileParser2 for ETParser {
    type Output = ETFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_et_str(&contents)
    }
}
