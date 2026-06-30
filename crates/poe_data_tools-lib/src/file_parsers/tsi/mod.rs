use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string2,
};

pub mod parser;
pub mod types;

use parser::parse_tsi_str;
pub use types::*;

pub struct TSIParser;

impl FileParser2 for TSIParser {
    type Output = TSIFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string2(bytes).to_parse_error()?;

        parse_tsi_str(&contents)
    }
}
