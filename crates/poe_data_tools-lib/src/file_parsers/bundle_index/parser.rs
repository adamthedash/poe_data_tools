use std::fmt::Display;

use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    binary::{le_u32, le_u64, length_repeat, length_take},
};

use super::types::*;
use crate::file_parsers::{bundle::parser::bundle, shared::winnow::WinnowParser};

fn bundle_info<'a>() -> impl WinnowParser<&'a [u8], BundleInfo> {
    winnow::trace!(
        "bundle_info",
        (
            length_take(le_u32).try_map(|bytes: &[_]| String::from_utf8(bytes.to_vec())),
            le_u32,
        )
            .map(|(name, uncompressed_size)| BundleInfo {
                name,
                uncompressed_size,
            })
    )
}

fn file_info<'a>() -> impl WinnowParser<&'a [u8], FileInfo> {
    winnow::trace!(
        "file_info",
        (le_u64, le_u32, le_u32, le_u32) //
            .map(|(hash, bundle_index, offset, size)| FileInfo {
                hash,
                bundle_index,
                offset,
                size,
            })
    )
}

fn path_rep<'a>() -> impl WinnowParser<&'a [u8], PathRep> {
    winnow::trace!(
        "path_rep",
        (le_u64, le_u32, le_u32, le_u32) //
            .map(|(hash, offset, size, recursive_size)| PathRep {
                hash,
                offset,
                size,
                recursive_size,
            })
    )
}

/// Wrapper for anyhow::Error that implements std Error so winnow can properly handle them
#[derive(Debug)]
pub struct AnyhowError(anyhow::Error);
impl std::error::Error for AnyhowError {}

impl Display for AnyhowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn parse_bundle_index_bytes(contents: &[u8]) -> Result<BundleIndexFile> {
    let mut parser = (
        length_repeat(le_u32, bundle_info()),
        length_repeat(le_u32, file_info()),
        length_repeat(le_u32, path_rep()),
        bundle().try_map(|b| b.read_all().map_err(AnyhowError)),
    )
        .map(|(bundles, files, paths, path_rep_bundle)| BundleIndexFile {
            bundles,
            files,
            paths,
            path_rep_bundle,
        });

    let file = parser
        .parse(contents)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(file)
}
