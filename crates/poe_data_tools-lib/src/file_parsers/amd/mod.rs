use crate::file_parsers::{
    FileParser2,
    error::{ParseErrorInner, Result},
};

pub mod parser;
pub mod types;
use parser::parse_amd_str;
use types::AMDFile;

pub struct AMDParser;

impl FileParser2 for AMDParser {
    type Output = AMDFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents =
            String::from_utf16le(bytes).map_err(|e| ParseErrorInner::Preprocessing(e.into()))?;

        parse_amd_str(&contents)
    }
}
