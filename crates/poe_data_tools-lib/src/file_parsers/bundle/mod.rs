use crate::file_parsers::{
    FileParser, VersionedFile,
    error::{AsParseError, Result},
};

pub mod parser;
pub mod types;
use types::BundleFile;
use winnow::Parser;

pub struct BundleParser;

impl FileParser for BundleParser {
    type Output = BundleFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parser::bundle().parse(bytes).to_parse_error()
    }
}

impl VersionedFile for BundleFile {
    fn version(&self) -> Option<u32> {
        None
    }
}
