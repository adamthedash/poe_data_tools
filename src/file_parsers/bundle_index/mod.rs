
use crate::file_parsers::{FileParser, VersionedResult};

pub mod parser;
pub mod types;
use parser::parse_bundle_index_bytes;
use types::BundleIndexFile;

pub struct BundleIndexParser;

impl FileParser for BundleIndexParser {
    type Output = BundleIndexFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        Ok(parse_bundle_index_bytes(bytes)?)
    }
}
