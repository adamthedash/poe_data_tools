use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, ParseErrorInner, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_epk_str;
use types::EPKFile;

pub struct EPKParser;

impl FileParser2 for EPKParser {
    type Output = EPKFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = if let Ok(s) = utf16_bom_to_string(bytes).to_parse_error() {
            s
        } else {
            String::from_utf8(bytes.to_vec())
                .map_err(|e| ParseErrorInner::Preprocessing(e.into()))?
        };

        parse_epk_str(&contents)
    }
}
