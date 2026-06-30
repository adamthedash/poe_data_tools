use crate::file_parsers::{
    FileParser, VersionedFile,
    error::{ParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_tst_str;
use types::TSTFile;

pub struct TSTParser;

impl FileParser for TSTParser {
    type Output = TSTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)
            .or_else(|_| String::from_utf16le(bytes))
            .map_err(ParseError::processing)?;

        parse_tst_str(&contents)
    }
}

impl VersionedFile for TSTFile {
    fn version(&self) -> Option<u32> {
        None
    }
}
