pub mod parser;
pub mod types;
use parser::parse_mat_str;
use types::MATFile;

use crate::file_parsers::{
    FileParser2,
    error::{ParseErrorInner, Result},
    shared::utf16_bom_to_string,
};

pub struct MATParser;

impl FileParser2 for MATParser {
    type Output = MATFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)
            .or_else(|_| String::from_utf16le(bytes))
            .map_err(|e| ParseErrorInner::Preprocessing(e.into()))?;

        parse_mat_str(&contents)
    }
}
