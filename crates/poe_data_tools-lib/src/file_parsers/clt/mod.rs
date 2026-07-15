use crate::file_parsers::{
    FileParser, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_clt_str;
use types::CLTFile;

pub struct CLTParser;

impl FileParser for CLTParser {
    type Output = CLTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_clt_str(&contents)
    }
}

impl VersionedFile for CLTFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
