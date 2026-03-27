pub mod parser;
pub mod types;
use parser::parse_mtd_str;
use types::MTDFile;

use crate::file_parsers::{FileParser, VersionedResult, shared::utf16_bom_to_string};

pub struct MTDParser;

impl FileParser for MTDParser {
    type Output = MTDFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_mtd_str(&contents)
    }
}
