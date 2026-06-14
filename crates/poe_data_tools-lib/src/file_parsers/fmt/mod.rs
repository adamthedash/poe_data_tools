pub mod parser;
pub mod types;
use parser::parse_fmt;
use types::*;

use crate::file_parsers::{FileParser, VersionedResult};

pub struct FMTParser;

impl FileParser for FMTParser {
    type Output = FMTFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        parse_fmt(bytes)
    }
}
