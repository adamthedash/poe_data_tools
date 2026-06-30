use crate::file_parsers::{
    FileParser2, VersionedFile,
    error::{AsParseError, Result},
};

pub mod parser;
pub mod types;
use types::BundleIndexFile;
use winnow::Parser;

pub struct BundleIndexParser;

impl FileParser2 for BundleIndexParser {
    type Output = BundleIndexFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parser::bundle_index_file().parse(bytes).to_parse_error()
    }
}

impl VersionedFile for BundleIndexFile {
    fn version(&self) -> Option<u32> {
        None
    }
}
