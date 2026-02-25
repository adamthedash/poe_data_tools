use anyhow::Result;

use crate::file_parsers::FileParser;

pub mod parser;
pub mod types;
use parser::parse_bundle_index_bytes;
use types::BundleIndexFile;

pub struct BundleIndexParser;

impl FileParser for BundleIndexParser {
    type Output = BundleIndexFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_bundle_index_bytes(bytes)
    }
}
