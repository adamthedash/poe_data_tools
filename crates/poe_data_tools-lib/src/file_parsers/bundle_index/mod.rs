use crate::file_parsers::{FileParser, VersionedFile, error::Result};

pub mod parser;
pub mod types;
use parser::parse_bundle_index_file_bytes;
use types::BundleIndexFile;

pub struct BundleIndexParser;

impl FileParser for BundleIndexParser {
    type Output = BundleIndexFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_bundle_index_file_bytes(bytes)
    }
}

impl VersionedFile for BundleIndexFile {
    fn version(&self) -> Option<u32> {
        None
    }
}
