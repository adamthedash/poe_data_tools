pub mod parser;
pub mod types;

use parser::parse_rs_str;
use types::*;

use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string2,
};

pub struct RSParser;

impl FileParser2 for RSParser {
    type Output = RSFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string2(bytes).to_parse_error()?;

        parse_rs_str(&contents)
    }
}
