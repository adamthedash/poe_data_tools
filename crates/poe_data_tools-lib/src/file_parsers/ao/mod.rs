pub mod parser;
pub mod types;

use parser::parse_ao_str;
use types::*;

use crate::file_parsers::{
    FileParser, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub struct AOParser;

impl FileParser for AOParser {
    type Output = AOFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_ao_str(contents.trim())
    }
}

impl VersionedFile for AOFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
