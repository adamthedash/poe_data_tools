use annotated_parser::{
    ForwardRef,
    parsers::{TakeArray, TakeVec},
    prelude::*,
};

use super::types::*;
use crate::file_parsers::{
    error::{AsParseError, Result},
    shared::annotated_parser::U8Parser,
};

#[derive(Debug, thiserror::Error)]
enum BundleError {
    #[error("invalid encoding identifier: {0}")]
    InvalidEncoding(u32),
}

fn first_file_encode() -> impl U8Parser<Output = FirstFileEncode> {
    u32::LE
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

fn header() -> impl U8Parser<Output = HeadPayload> {
    (
        TakeArray::<12>,
        first_file_encode(),
        TakeArray::<4>,
        u64::LE.trace("uncompressed_size"),
        u64::LE.trace("total_payload_size"),
        u32::LE.trace("block_count"),
        u32::LE.trace("uncompressed_block_gr"),
        TakeArray::<16>,
    )
        .map_silent(
            |(
                unk1,
                first_file_encode,
                unk2,
                uncompressed_size,
                total_payload_size,
                block_count,
                uncompressed_block_granularity,
                unk3,
            )| HeadPayload {
                unk1,
                first_file_encode,
                unk2,
                uncompressed_size,
                total_payload_size,
                block_count,
                uncompressed_block_granularity,
                unk3,
            },
        )
        .trace("header")
}

fn blocks(block_count: ForwardRef<u32>) -> impl U8Parser<Output = Vec<Vec<u8>>> {
    let block_sizes = u32::LE.repeat_vec(block_count).trace("block_sizes").store();

    let block_size = ForwardRef::new_source();
    let blocks = TakeVec::new(block_size.clone()).parameterize(block_sizes.output(), block_size);

    (block_sizes, blocks)
        .map_silent(|(_, blocks)| blocks)
        .trace("blocks")
}

pub fn bundle() -> impl U8Parser<Output = BundleFile> {
    let head = header().store();
    let blocks = blocks(head.output().map(|h| h.block_count));

    (head, blocks)
        .map_silent(|(head, blocks)| BundleFile { head, blocks })
        .trace("bundle")
}

pub fn parse_bundle_bytes(mut input: &[u8]) -> Result<BundleFile> {
    let mut parser = bundle();

    let (bundle_file, _) = parser.parse(&mut input).to_parse_error()?;

    Ok(bundle_file)
}
