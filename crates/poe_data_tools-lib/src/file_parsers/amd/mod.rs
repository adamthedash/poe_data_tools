use crate::file_parsers::{
    FileParser, VersionedFile,
    error::{ParseError, Result},
};

pub mod parser;
pub mod types;
use parser::parse_amd_str;
use types::AMDFile;

pub struct AMDParser;

impl FileParser for AMDParser {
    type Output = AMDFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = String::from_utf16le(bytes).map_err(ParseError::processing)?;

        parse_amd_str(&contents)
    }
}

impl VersionedFile for AMDFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
