use crate::file_parsers::{
    FileParser2, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_dlp_str;
use types::*;

pub struct DLPParser;

impl FileParser2 for DLPParser {
    type Output = DLPFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_dlp_str(&contents)
    }
}

impl VersionedFile for DLPFile {
    fn version(&self) -> Option<u32> {
        self.version
    }
}
