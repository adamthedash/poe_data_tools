use annotated_parser::{combinators::LengthRepeat, parsers::TakeVec, prelude::*};

use super::types::*;
use crate::file_parsers::{
    bundle::parser::bundle,
    error::{AsParseError, Result},
    shared::annotated_parser::U8Parser,
};

fn bundle_info() -> impl U8Parser<Output = BundleInfo> {
    let name_length = u32::LE.store();
    let name = TakeVec::new(name_length.output())
        .try_map(String::from_utf8)
        .trace("name");

    (name_length, name, u32::LE)
        .map_silent(|(_, name, uncompressed_size)| BundleInfo {
            name,
            uncompressed_size,
        })
        .trace("bundle_info")
}

fn file_info() -> impl U8Parser<Output = FileInfo> {
    (u64::LE, u32::LE, u32::LE, u32::LE)
        .map_silent(|(hash, bundle_index, offset, size)| FileInfo {
            hash,
            bundle_index,
            offset,
            size,
        })
        .trace("file_info")
}

fn path_rep() -> impl U8Parser<Output = PathRep> {
    (u64::LE, u32::LE, u32::LE, u32::LE)
        .map_silent(|(hash, offset, size, recursive_size)| PathRep {
            hash,
            offset,
            size,
            recursive_size,
        })
        .trace("path_rep")
}

pub fn bundle_index_file() -> impl U8Parser<Output = BundleIndexFile> {
    (
        LengthRepeat::new(u32::LE, bundle_info()),
        LengthRepeat::new(u32::LE, file_info()),
        LengthRepeat::new(u32::LE, path_rep()),
        bundle().try_map(|b| b.read_all()),
    )
        .map_silent(|(bundles, files, paths, path_rep_bundle)| BundleIndexFile {
            bundles,
            files,
            paths,
            path_rep_bundle,
        })
        .trace("bundle_index_file")
}

pub fn parse_bundle_index_file_bytes(mut input: &[u8]) -> Result<BundleIndexFile> {
    let mut parser = bundle_index_file();

    let (bundle_index, _) = parser.parse(&mut input).to_parse_error()?;

    Ok(bundle_index)
}
