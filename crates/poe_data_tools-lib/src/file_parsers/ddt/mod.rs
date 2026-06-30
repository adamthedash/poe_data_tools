pub mod parser;
pub mod types;

use parser::parse_ddt_str;
use types::*;

use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string2,
};

pub struct DDTParser;

impl FileParser2 for DDTParser {
    type Output = DDTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string2(bytes).to_parse_error()?;

        parse_ddt_str(&contents)
    }
}
