use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string,
};

pub mod parser;
pub mod types;
use parser::parse_arm_str;
use types::ARMFile;

pub struct ARMParser;

impl FileParser2 for ARMParser {
    type Output = ARMFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes).to_parse_error()?;

        parse_arm_str(&contents)
    }
}
