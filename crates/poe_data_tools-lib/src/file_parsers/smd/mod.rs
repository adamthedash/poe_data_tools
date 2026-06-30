pub mod parser;
pub mod types;

use parser::parse_smd;
use types::SMDFile;

use crate::file_parsers::{FileParser2, error::Result};

pub struct SMDParser;

impl FileParser2 for SMDParser {
    type Output = SMDFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_smd(bytes)
    }
}
