use crate::file_parsers::{
    FileParser2, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_cht_str;
use types::*;

pub struct CHTParser;

impl FileParser2 for CHTParser {
    type Output = CHTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_cht_str(&contents)
    }
}

impl VersionedFile for CHTFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
