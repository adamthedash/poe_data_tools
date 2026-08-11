use crate::file_parsers::{FileParser, VersionedFile, error::Result};

pub mod parser;
pub mod types;
use parser::parse_bundle_bytes;
use types::BundleFile;

pub struct BundleParser;

impl FileParser for BundleParser {
    type Output = BundleFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_bundle_bytes(bytes)
    }
}

impl VersionedFile for BundleFile {
    fn version(&self) -> Option<u32> {
        None
    }
}
