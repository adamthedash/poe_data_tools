use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
};

pub mod parser;
pub mod types;
use types::BundleFile;
use winnow::Parser;

pub struct BundleParser;

impl FileParser2 for BundleParser {
    type Output = BundleFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parser::bundle().parse(bytes).to_parse_error()
    }
}
