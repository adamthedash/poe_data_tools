use anyhow::anyhow;
use winnow::{binary::le_u8, error::ContextError};

use super::types::*;
use crate::file_parsers::{VersionedResult, VersionedResultExt};

pub fn parse_smd(mut contents: &[u8]) -> VersionedResult<SMDFile> {
    let version =
        le_u8(&mut contents).map_err(|e: ContextError| anyhow!("Failed to parse file: {e:?}"))?;

    Err(anyhow!("blah")).with_version(Some(version as u32))
}
