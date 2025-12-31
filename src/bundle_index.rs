use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use nom::{
    bytes::complete::take,
    combinator::rest,
    multi::count,
    number::complete::{le_u32, le_u64},
    IResult,
};
use url::Url;

use crate::bundle::{fetch_bundle_content, load_bundle_content, parse_bundle};

#[derive(Debug)]
pub struct BundleInfo {
    /// Partial path of this bundle file
    pub name: String,
    /// Total size of contents once uncompressed
    pub uncompressed_size: u32,
}

/// Pointer to where within the bundles a given virtual file lives
#[derive(Debug)]
pub struct FileInfo {
    /// MurmurHash64 of the virtual file path
    pub hash: u64,
    /// Which bundle the file lives in
    pub bundle_index: u32,
    /// Offset in uncompressed bundle contents
    pub offset: u32,
    /// Uncompressed file size
    pub size: u32,
}

/// Pointer into compact path storage section of bundle index for one "chunk" of paths.  
#[derive(Debug)]
pub struct PathRep {
    /// MurmurHash64 of this file path
    pub hash: u64,
    /// Offset in uncompressed data
    pub offset: u32,
    /// Uncompressed size
    pub size: u32,
    pub recursive_size: u32,
}

/// Index of all the other bundle files and virtual files contained in them
#[derive(Debug)]
pub struct BundleIndex {
    /// List of all bundles
    pub bundles: Vec<BundleInfo>,
    /// List of virtual file sizes and locations
    pub files: Vec<FileInfo>,
    /// List of virtual file paths
    pub paths: Vec<PathRep>,
    /// Compact path data
    pub path_rep_bundle: Bytes,
}

/// Parser for a UTF-8 string of given length
fn parse_string(input: &[u8], length: u32) -> IResult<&[u8], String> {
    let (input, data) = take(length)(input)?;
    let string = String::from_utf8_lossy(data).to_string();
    Ok((input, string))
}

/// Parser for a Bundle
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

/// Parser for a vector of Bundles
fn parse_bundles(input: &[u8]) -> IResult<&[u8], Vec<BundleInfo>> {
    let (input, bundle_count) = le_u32(input)?;
    count(parse_bundle_info, bundle_count as usize)(input)
}

/// Parser for a FileInfo
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

/// Parser for a vector of PathRep
fn parse_path_reps(input: &[u8]) -> IResult<&[u8], Vec<PathRep>> {
    let (input, path_count) = le_u32(input)?;
    count(parse_path_rep, path_count as usize)(input)
}

/// Parser for the entire BundleIndex
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
            path_rep_bundle: path_rep_bundle.read_all(),
        },
    ))
}

/// Load the bundle index file from disk.  
///
/// # Examples
/// ```
/// load_index_file(&PathBuf::from("<steam_poe_folder>/Bundles2/_.index.bin")).unwrap();
/// ```
pub fn load_index_file(path: &Path) -> Result<BundleIndex> {
    let index_content = load_bundle_content(path)
        .context("Failed to read bundle index")?
        .read_all();
    let (_, index) = parse_bundle_index(&index_content)
        .map_err(|_| anyhow!("Failed to parse bundle as index"))?;
    Ok(index)
}

/// Fetch the bundle index file from the CDN (or cache)
///
/// # Examples
/// ```
/// let base_url = Url::parse("https://patch-poe2.poecdn.com/4.4.0.3.9/").unwrap();
/// let cache_path = PathBuf::from(".cache");
/// let bundle_path = PathBuf::from("Bundles2/_.index.bin");
/// let bundle = fetch_index_file(&base_url, &cache_path, &bundle_path).unwrap();
/// ```
pub fn fetch_index_file(base_url: &Url, cache_dir: &Path, path: &Path) -> Result<BundleIndex> {
    let index_content = fetch_bundle_content(base_url, cache_dir, path)
        .context("Failed to fetch bundle index")?
        .read_all();
    let (_, index) = parse_bundle_index(&index_content)
        .map_err(|_| anyhow!("Failed to parse bundle as index"))?;
    Ok(index)
}
