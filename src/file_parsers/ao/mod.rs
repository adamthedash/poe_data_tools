pub mod parser;
pub mod types;

use parser::parse_ao_str;
use types::*;

use crate::file_parsers::{FileParser, VersionedResult, shared::utf16_bom_to_string};

pub struct AOParser;

impl FileParser for AOParser {
    type Output = AOFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_ao_str(contents.trim())
    }
}
