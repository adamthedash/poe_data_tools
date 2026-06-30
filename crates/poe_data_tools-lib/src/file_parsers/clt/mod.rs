use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string2,
};

pub mod parser;
pub mod types;
use parser::parse_clt_str;
use types::CLTFile;

pub struct CLTParser;

impl FileParser2 for CLTParser {
    type Output = CLTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string2(bytes).to_parse_error()?;

        parse_clt_str(&contents)
    }
}
