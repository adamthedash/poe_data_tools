use crate::file_parsers::{
    FileParser2, VersionedFile,
    error::{AsParseError, Result},
};

pub mod parser;
pub mod types;
use parser::dat;
use types::DatFile;
use winnow::Parser;

pub struct DatParser;

impl FileParser2 for DatParser {
    type Output = DatFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        dat().parse(bytes).to_parse_error()
    }
}

impl VersionedFile for DatFile {
    fn version(&self) -> Option<u32> {
        None
    }
}
