use crate::file_parsers::{
    FileParser2, VersionedFile,
    error::{ParseError, Result},
};

pub mod parser;
pub mod types;
use parser::parse_tgt_str;
use types::*;

pub struct TGTParser;

impl FileParser2 for TGTParser {
    type Output = TGTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = String::from_utf16le(bytes).map_err(ParseError::processing)?;

        parse_tgt_str(&contents)
    }
}

impl VersionedFile for TGTFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
