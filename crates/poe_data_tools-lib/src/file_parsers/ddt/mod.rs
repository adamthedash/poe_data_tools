pub mod parser;
pub mod types;

use parser::parse_ddt_str;
use types::*;

use crate::file_parsers::{
    FileParser2, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub struct DDTParser;

impl FileParser2 for DDTParser {
    type Output = DDTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_ddt_str(&contents)
    }
}

impl VersionedFile for DDTFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
