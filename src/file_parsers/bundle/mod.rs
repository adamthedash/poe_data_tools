use crate::file_parsers::{FileParser, VersionedResult, VersionedResultExt};

pub mod parser;
pub mod types;
use parser::parse_bundle_bytes;
use types::BundleFile;

pub struct BundleParser;

impl FileParser for BundleParser {
    type Output = BundleFile;

    fn parse(&self, bytes: &[u8]) -> VersionedResult<Self::Output> {
        parse_bundle_bytes(bytes).unversioned()
    }
}
