use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string2,
};

pub mod parser;
pub mod types;
use parser::parse_trl_str;
use types::*;

pub struct TRLParser;

impl FileParser2 for TRLParser {
    type Output = TRLFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string2(bytes).to_parse_error()?;

        parse_trl_str(&contents)
    }
}
