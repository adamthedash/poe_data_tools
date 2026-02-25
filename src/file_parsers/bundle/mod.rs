use anyhow::Result;

use crate::file_parsers::FileParser;

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
