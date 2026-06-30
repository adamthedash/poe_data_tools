use crate::file_parsers::{
    FileParser2, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_ecf_str;
use types::*;

pub struct ECFParser;

impl FileParser2 for ECFParser {
    type Output = EcfFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_ecf_str(&contents)
    }
}

impl VersionedFile for EcfFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
