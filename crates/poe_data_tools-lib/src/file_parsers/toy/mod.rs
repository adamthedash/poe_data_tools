use crate::file_parsers::{
    FileParser2, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_toy_str;
use types::*;

pub struct TOYParser;

impl FileParser2 for TOYParser {
    type Output = TOYFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_toy_str(&contents)
    }
}

impl VersionedFile for TOYFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
