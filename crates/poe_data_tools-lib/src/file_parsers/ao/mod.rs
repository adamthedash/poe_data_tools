pub mod parser;
pub mod types;

use parser::parse_ao_str;
use types::*;

use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub struct AOParser;

impl FileParser2 for AOParser {
    type Output = AOFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_ao_str(contents.trim())
    }
}
