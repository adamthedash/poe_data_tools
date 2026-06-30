pub mod parser;
pub mod types;
use parser::parse_mtd_str;
use types::MTDFile;

use crate::file_parsers::{
    FileParser2, VersionedFile,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub struct MTDParser;

impl FileParser2 for MTDParser {
    type Output = MTDFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_mtd_str(&contents)
    }
}

impl VersionedFile for MTDFile {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }
}
