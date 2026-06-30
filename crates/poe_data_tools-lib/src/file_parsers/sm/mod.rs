use crate::file_parsers::{
    FileParser, VersionedFile,
    error::{ParseError, Result},
};

pub mod parser;
pub mod types;
use parser::parse_sm_str;
use types::*;

pub struct SMParser;

impl FileParser for SMParser {
    type Output = SMFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = String::from_utf16le(bytes).map_err(ParseError::processing)?;

        parse_sm_str(&contents)
    }
}

impl VersionedFile for SMFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
