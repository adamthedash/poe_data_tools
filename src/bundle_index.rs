use std::path::Path;

use nom::{
    bytes::complete::take,
    combinator::rest,
    multi::count,
    number::complete::{le_u32, le_u64},
    IResult,
};

use crate::bundle::{load_bundle_content, parse_bundle};

#[derive(Debug)]
pub struct BundleInfo {
    pub name: String,
    pub uncompressed_size: u32,
}

#[derive(Debug)]
pub struct FileInfo {
    pub hash: u64,
    pub bundle_index: u32,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug)]
pub struct PathRep {
    pub hash: u64,
    pub offset: u32,
    pub size: u32,
    pub recursive_size: u32,
}

#[derive(Debug)]
pub struct BundleIndex {
    pub bundles: Vec<BundleInfo>,
    pub files: Vec<FileInfo>,
    pub paths: Vec<PathRep>,
    pub path_rep_bundle: Vec<u8>,
}

// Parser for a UTF-8 string of given length
fn parse_string(input: &[u8], length: u32) -> IResult<&[u8], String> {
    let (input, data) = take(length)(input)?;
    let string = String::from_utf8_lossy(data).to_string();
    Ok((input, string))
}

// Parser for a Bundle
fn parse_bundle_info(input: &[u8]) -> IResult<&[u8], BundleInfo> {
    let (input, name_length) = le_u32(input)?;
    let (input, name) = parse_string(input, name_length)?;
    let (input, uncompressed_size) = le_u32(input)?;
    Ok((
        input,
        BundleInfo {
            name,
            uncompressed_size,
        },
    ))
}

// Parser for a vector of Bundles
fn parse_bundles(input: &[u8]) -> IResult<&[u8], Vec<BundleInfo>> {
    let (input, bundle_count) = le_u32(input)?;
    count(parse_bundle_info, bundle_count as usize)(input)
}

// Parser for a FileInfo
fn parse_file_info(input: &[u8]) -> IResult<&[u8], FileInfo> {
    let (input, hash) = le_u64(input)?;
    let (input, bundle_index) = le_u32(input)?;
    let (input, offset) = le_u32(input)?;
    let (input, size) = le_u32(input)?;
    Ok((
        input,
        FileInfo {
            hash,
            bundle_index,
            offset,
            size,
        },
    ))
}

// Parser for a vector of FileInfo
fn parse_file_infos(input: &[u8]) -> IResult<&[u8], Vec<FileInfo>> {
    let (input, file_count) = le_u32(input)?;
    count(parse_file_info, file_count as usize)(input)
}

// Parser for a PathRep
fn parse_path_rep(input: &[u8]) -> IResult<&[u8], PathRep> {
    let (input, hash) = le_u64(input)?;
    let (input, offset) = le_u32(input)?;
    let (input, size) = le_u32(input)?;
    let (input, recursive_size) = le_u32(input)?;
    Ok((
        input,
        PathRep {
            hash,
            offset,
            size,
            recursive_size,
        },
    ))
}

// Parser for a vector of PathRep
fn parse_path_reps(input: &[u8]) -> IResult<&[u8], Vec<PathRep>> {
    let (input, path_count) = le_u32(input)?;
    count(parse_path_rep, path_count as usize)(input)
}

// Parser for the entire BundleIndex
pub fn parse_bundle_index(input: &[u8]) -> IResult<&[u8], BundleIndex> {
    let (input, bundles) = parse_bundles(input)?;
    let (input, files) = parse_file_infos(input)?;
    let (input, paths) = parse_path_reps(input)?;
    let (input, path_rep_bundle) = rest(input)?;
    let (_, path_rep_bundle) = parse_bundle(path_rep_bundle)?;

    Ok((
        input,
        BundleIndex {
            bundles,
            files,
            paths,
            path_rep_bundle: path_rep_bundle.read_content(),
        },
    ))
}

/// Load an index file from disk
pub fn load_index_file(path: &Path) -> BundleIndex {
    let index_content = load_bundle_content(path);
    let (_, index) = parse_bundle_index(&index_content).expect("Failed to parse bundle as index");

    index
}
