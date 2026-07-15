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

pub fn bundle_index_file<'a>() -> impl WinnowParser<&'a [u8], BundleIndexFile> {
    let parser = (
        length_repeat(le_u32, bundle_info()),
        length_repeat(le_u32, file_info()),
        length_repeat(le_u32, path_rep()),
        bundle().try_map(|b| b.read_all()),
    )
        .map(|(bundles, files, paths, path_rep_bundle)| BundleIndexFile {
            bundles,
            files,
            paths,
            path_rep_bundle,
        });

    winnow::trace!("bundle_index_file", parser)
}
