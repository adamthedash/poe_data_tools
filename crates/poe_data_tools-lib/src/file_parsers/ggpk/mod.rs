use crate::file_parsers::VersionedFile;

pub mod parser;
pub mod types;
use types::GGPKFile;

impl VersionedFile for GGPKFile {
    fn version(&self) -> Option<u32> {
        None
    }
}
