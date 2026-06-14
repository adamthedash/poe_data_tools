use crate::file_parsers::{FileParser, VersionedResult, VersionedResultExt};

pub mod parser;
pub mod types;
use parser::parse_dat_bytes;
use types::DatFile;

pub struct DatParser;

impl FileParser for DatParser {
    type Output = DatFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        parse_dat_bytes(bytes).unversioned()
    }
}
