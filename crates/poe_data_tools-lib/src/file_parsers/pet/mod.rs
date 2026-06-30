use crate::file_parsers::{
    FileParser2,
    error::{AsParseError, Result},
    shared::utf16_bom_to_string2,
};

pub mod parser;
pub mod types;
use parser::parse_pet_str;
use types::*;

pub struct PETParser;

impl FileParser2 for PETParser {
    type Output = PETFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string2(bytes).to_parse_error()?;

        parse_pet_str(&contents)
    }
}
