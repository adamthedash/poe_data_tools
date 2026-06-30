use crate::file_parsers::{
    FileParser, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_tmo_str;
use types::*;

pub struct TMOParser;

impl FileParser for TMOParser {
    type Output = TMOFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_tmo_str(&contents)
    }
}

impl VersionedFile for TMOFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
