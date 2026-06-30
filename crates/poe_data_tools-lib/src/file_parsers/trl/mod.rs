use crate::file_parsers::{
    FileParser2, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_trl_str;
use types::*;

pub struct TRLParser;

impl FileParser2 for TRLParser {
    type Output = TRLFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_trl_str(&contents)
    }
}

impl VersionedFile for TRLFile {
    fn version(&self) -> Option<u32> {
        self.version
    }
}
