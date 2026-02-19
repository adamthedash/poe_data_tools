use crate::file_parsers::{FileParser, shared::utf16_bom_to_string};

pub mod parser;
pub mod types;
use parser::parse_arm_str;
use types::ARMFile;

pub struct ARMParser;

impl FileParser for ARMParser {
    type Output = ARMFile;

    fn parse(&self, bytes: &[u8]) -> anyhow::Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_arm_str(&contents)
    }
}
