use std::{fs, path::Path};

use nom::{
    bytes::complete::take,
    multi::count,
    number::complete::{le_u32, le_u64},
    IResult,
};
use oozextract::Extractor;

use crate::bundle_loader::CDNLoader;

/// Encoded as a u32
#[derive(Debug)]
pub enum FirstFileEncode {
    Kraken6 = 8,
    MermaidA = 9,
    Bitknit = 12,
    LeviathanC = 13,
}

impl FirstFileEncode {
    fn from_u32(value: u32) -> Option<Self> {
        match value {
            8 => Some(Self::Kraken6),
            9 => Some(Self::MermaidA),
            12 => Some(Self::Bitknit),
            13 => Some(Self::LeviathanC),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct HeadPayload {
    pub first_file_encode: FirstFileEncode,
    pub uncompressed_size: u64,
    pub total_payload_size: u64,
    pub uncompressed_block_granularity: u32,
}

#[derive(Debug)]
pub struct Bundle {
    pub head: HeadPayload,
    pub blocks: Vec<Vec<u8>>,
}

impl Bundle {
    /// Return the entire conent of the bundle
    /// todo: decode blocks in parallel
    ///     Also return a result instead of panicing
    pub fn read_content(&self) -> Vec<u8> {
        let mut ext = Extractor::new();
        let mut buf = vec![0; self.head.uncompressed_block_granularity as usize];

        let bundle_conent = self
            .blocks
            .iter()
            .enumerate()
            .flat_map(|(i, b)| {
                let decompressed_length = if i == self.blocks.len() - 1 {
                    self.head.uncompressed_size as usize - 256 * 1024 * (self.blocks.len() - 1)
                } else {
                    self.head.uncompressed_block_granularity as usize
                };

                ext.read_from_slice(b, &mut buf[..decompressed_length])
                    .expect("Failed to decompress bundle block");

                buf[..decompressed_length].to_vec()
            })
            .collect::<Vec<_>>();

        bundle_conent
    }
}

// Parser for FirstFileEncode
fn parse_first_file_encode(input: &[u8]) -> IResult<&[u8], FirstFileEncode> {
    let (input, value) = le_u32(input)?;
    match FirstFileEncode::from_u32(value) {
        Some(encode) => Ok((input, encode)),
        None => Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Alt,
        ))),
    }
}

// Parser for HeadPayload
fn parse_head_payload(input: &[u8]) -> IResult<&[u8], (HeadPayload, Vec<u32>)> {
    let (input, _) = take(12usize)(input)?; // Skip bytes 0-12
    let (input, first_file_encode) = parse_first_file_encode(input)?;
    let (input, _) = take(4usize)(input)?; // Skip bytes 16-20
    let (input, uncompressed_size) = le_u64(input)?;
    let (input, total_payload_size) = le_u64(input)?;
    let (input, block_count) = le_u32(input)?;
    let (input, uncompressed_block_granularity) = le_u32(input)?;
    let (input, _) = take(16usize)(input)?; // Skip bytes 44-60

    // Read block sizes (block_count u32s)
    let (input, block_sizes) = count(le_u32, block_count as usize)(input)?;

    Ok((
        input,
        (
            HeadPayload {
                first_file_encode,
                uncompressed_size,
                total_payload_size,
                uncompressed_block_granularity,
            },
            block_sizes,
        ),
    ))
}

// Parser for blocks
fn parse_blocks<'a>(input: &'a [u8], block_sizes: &[u32]) -> IResult<&'a [u8], Vec<Vec<u8>>> {
    let mut remaining_input = input;
    let mut blocks = Vec::new();

    for &block_size in block_sizes {
        let (input, block_data) = take(block_size)(remaining_input)?;
        blocks.push(block_data.to_vec());
        remaining_input = input;
    }

    Ok((remaining_input, blocks))
}

// Parser for Bundle
pub fn parse_bundle(input: &[u8]) -> IResult<&[u8], Bundle> {
    let (input, (head, block_sizes)) = parse_head_payload(input)?;
    let (input, blocks) = parse_blocks(input, &block_sizes)?;
    Ok((input, Bundle { head, blocks }))
}

/// Load a bundle file from disk
pub fn load_bundle_content(path: &Path) -> Vec<u8> {
    // todo: figure how to properly do error propogation with nom
    let bundle_content = fs::read(path).expect("Failed to read bundle file");

    let (_, bundle) = parse_bundle(&bundle_content).expect("Failed to parse bundle");
    bundle.read_content()
}

// Fetch a bundle file from the CDN (or cache)
pub fn fetch_bundle_content(patch: &str, cache_dir: &Path, path: &Path) -> Vec<u8> {
    let bundle_content = CDNLoader::new(patch, cache_dir.to_str().unwrap())
        .load(path)
        .expect("Failed to load bundle");
    let (_, bundle) = parse_bundle(&bundle_content).expect("Failed to parse bundle");
    bundle.read_content()
}
