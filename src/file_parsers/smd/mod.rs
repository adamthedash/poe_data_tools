pub mod parser;
pub mod types;

use types::SMDFile;

use crate::file_parsers::{FileParser, VersionedResult};

pub struct SMDParser;

impl FileParser for SMDParser {
    type Output = SMDFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        parse_smd(bytes)
    }
}
