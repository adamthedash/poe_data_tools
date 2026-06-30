use crate::file_parsers::{
    FileParser, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_gft_str;
use types::*;

pub struct GFTParser;

impl FileParser for GFTParser {
    type Output = GFTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_gft_str(&contents)
    }
}

impl VersionedFile for GFTFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
