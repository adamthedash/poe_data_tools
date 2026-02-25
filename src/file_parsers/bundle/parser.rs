use std::fmt::Display;

use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    binary::{le_u32, le_u64},
    combinator::{repeat, seq},
    token::take,
};

use super::types::*;
use crate::file_parsers::shared::winnow::{TraceHelper, WinnowParser};

#[derive(Debug)]
enum BundleError {
    InvalidEncoding(u32),
}
impl Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::InvalidEncoding(x) => write!(f, "Invalid encoding identifier: {x}"),
        }
    }
}

impl std::error::Error for BundleError {}

fn first_file_encode<'a>() -> impl WinnowParser<&'a [u8], FirstFileEncode> {
    le_u32
        .try_map(|x| {
            use FirstFileEncode::*;
            let ffe = match x {
                8 => Kraken6,
                9 => MermaidA,
                12 => Bitknit,
                13 => LeviathanC,
                x => {
                    return Err(BundleError::InvalidEncoding(x));
                }
            };

            Ok(ffe)
        })
        .trace("first_file_encode")
}

fn head_payload<'a>() -> impl WinnowParser<&'a [u8], (HeadPayload, u32)> {
    seq!((
        _: take(12_usize),
        first_file_encode(),
        _: take(4_usize),
        le_u64,
        le_u64,
        le_u32,
        le_u32,
        _: take(16_usize),
    ))
    .map(
        |(
            first_file_encode,
            uncompressed_size,
            total_payload_size,
            block_count,
            uncompressed_block_granularity,
        )| {
            let head_payload = HeadPayload {
                first_file_encode,
                uncompressed_size,
                total_payload_size,
                uncompressed_block_granularity,
            };

            (head_payload, block_count)
        },
    )
    .trace("head_payload")
}

fn blocks<'a>(block_count: u32) -> impl WinnowParser<&'a [u8], Vec<Vec<u8>>> {
    let parser = move |input: &mut &[u8]| -> winnow::Result<_> {
        let block_sizes: Vec<_> = repeat(block_count as usize, le_u32).parse_next(input)?;

        let mut blocks = vec![];
        for size in block_sizes {
            let block = take(size).parse_next(input)?;
            blocks.push(block.to_vec());
        }

        Ok(blocks)
    };

    parser.trace("blocks")
}

pub fn bundle<'a>() -> impl WinnowParser<&'a [u8], BundleFile> {
    let parser = |input: &mut &[u8]| {
        let (head, block_count) = head_payload().parse_next(input)?;

        let blocks = blocks(block_count).parse_next(input)?;

        let bundle = BundleFile { head, blocks };

        Ok(bundle)
    };

    parser.trace("bundle")
}

pub fn parse_bundle_bytes(contents: &[u8]) -> Result<BundleFile> {
    bundle()
        .parse(contents)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
}
