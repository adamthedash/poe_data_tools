pub mod parser;
pub mod types;
use parser::parse_ast_bytes;
use types::ASTFile;

use crate::file_parsers::{FileParser, VersionedFile, error::Result};

pub struct ASTParser;

impl FileParser for ASTParser {
    type Output = ASTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        parse_ast_bytes(bytes)
    }
}

impl VersionedFile for ASTFile {
    fn version(&self) -> Option<u32> {
        Some(self.header.version as u32)
    }
}
