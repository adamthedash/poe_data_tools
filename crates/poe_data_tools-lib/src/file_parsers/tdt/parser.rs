use annotated_parser::{
    ForwardRef,
    combinators::{LengthRepeat, TakeTillExc},
    parsers::{EoF, byte::Bool},
    prelude::*,
};

use super::types::*;
use crate::file_parsers::{
    error::{AsParseError, ParseResultEx, Result},
    shared::annotated_parser::{U8Parser, empty_u8, take_arr_u8},
};

fn strings_section() -> impl U8Parser<Output = String> {
    LengthRepeat::new(u32::LE, u16::LE)
        .try_map(|data| String::from_utf16(&data))
        .trace("strings_section")
}

fn indexed_string(strings: ForwardRef<String>) -> impl U8Parser<Output = Option<String>> {
    u32::LE
        .try_map(move |i| {
            let strings = strings.get();

            if i == u32::MAX {
                return Ok(None);
            }

            if i as usize >= strings.len() {
                return Err("String index out of bounds");
            }

            let Some(end) = strings[i as usize..]
                .char_indices()
                .find_map(|(i, c)| (c == '\0').then_some(i))
            else {
                return Err("Did not find null terminator");
            };

            let string = &strings[i as usize..i as usize + end];

            if string.is_empty() {
                return Ok(None);
            }

            Ok(Some(string.to_owned()))
        })
        .trace("indexed_string")
}

fn header(strings: ForwardRef<String>) -> impl U8Parser<Output = Header> {
    let tdt_file = indexed_string(strings.clone()).store().trace("tdt_file");

    let flags = u8::LE
        .run_if(tdt_file.output().map(Option::is_some))
        .trace("flags")
        .store();

    let unk_u16 = u16::LE
        .run_if(
            flags
                .output()
                .map(|f| f.is_some_and(|f| [1, 2].contains(&f))),
        )
        .trace("unk_u16");

    let tgt_file = indexed_string(strings.clone())
        .run_if(
            flags
                .output()
                .map(|f| f.is_none_or(|f| [1, 2, 9].contains(&f))),
        )
        .trace("tgt_file");

    let tag = indexed_string(strings)
        .run_if(
            flags
                .output()
                .map(|f| f.is_none_or(|f| [8, 9, 0xa, 0x1a, 0x18, 0x2a, 0x1c, 0x1e].contains(&f))),
        )
        .trace("tag");

    (tdt_file, flags, unk_u16, tgt_file, tag)
        .map_silent(|(tdt_file, flags, num, tgt_file, tag)| Header {
            tdt_file,
            flags,
            num,
            tgt_file: tgt_file.flatten(),
            tag: tag.flatten(),
        })
        .trace("header")
}

fn tail(strings: ForwardRef<String>, version: ForwardRef<u32>) -> impl U8Parser<Output = Tail> {
    let side_ets = indexed_string(strings.clone())
        .repeat::<4>()
        .trace("side_ets");
    let dims = u8::LE.repeat::<2>().trace("dims1").store();
    let side_gts = indexed_string(strings.clone())
        .repeat::<4>()
        .trace("side_gts")
        .store();

    let dims2 = u8::LE.repeat::<2>().trace("dims2");
    let dims3 = u8::LE.repeat::<2>().trace("dims3");
    let side_offsets = u8::LE.repeat::<8>().trace("side_offsets");
    let flags1 = u8::LE.trace("flags1").store();
    let flags2 = u8::LE.trace("flags2").store();

    let num_subtiles = u16::LE
        .trace("num_subtiles")
        .verify({
            let dims = dims.output();
            move |n| {
                let dims = dims.get().iter().map(|d| *d as u16).product::<u16>();

                dims == *n
            }
        })
        .trace("same_as_dims");

    let flags_rest = TakeTillExc::new(num_subtiles.clone()).trace("flags_rest");

    let subtiles =
        LengthRepeat::new(num_subtiles, subtile(flags2.output(), version)).trace("subtiles");

    let trailing = trailing(strings, flags1.output(), dims.output(), side_gts.output());

    let parser = (
        side_ets,
        dims,
        side_gts,
        dims2,
        dims3,
        side_offsets,
        flags1,
        flags2,
        flags_rest,
        subtiles,
        trailing,
    )
        .map_silent(
            |(
                side_ets,
                dims,
                side_gts,
                dims2,
                dims3,
                side_offsets,
                flags1,
                flags2,
                flags_rest,
                subtiles,
                trailing,
            )| {
                Tail {
                    side_ets,
                    dims,
                    side_gts,
                    dims2,
                    dims3,
                    side_offsets,
                    flags1,
                    flags2,
                    flags_rest,
                    subtiles,
                    trailing,
                }
            },
        )
        .trace("tail");

    parser.checkpoint()
}

fn fixed_block() -> impl U8Parser<Output = [u8; 23 * 23]> {
    take_arr_u8::<{ 23 * 23 }>().trace("fixed_block")
}

fn vary_block() -> impl U8Parser<Output = Vec<u8>> {
    LengthRepeat::new(u16::LE, u8::LE).trace("vary_block")
}

fn subtile(
    flags2: ForwardRef<u8>,
    version: ForwardRef<u32>,
) -> impl U8Parser<Output = super::types::Subtile> {
    let kind = u8::LE.store().trace("kind");

    let value = u8::LE.trace("value");

    let value_v10 = u8::LE.run_if(version.map(|v| *v >= 10));

    let bytes = (
        take_arr_u8::<2>().map_silent(Vec::from),
        take_arr_u8::<5>().map_silent(Vec::from),
        empty_u8().map_silent(|_| vec![]),
    )
        .dispatch(flags2.map(|f| {
            let index = match f {
                // 4 => 0,
                4 => 2,
                5 | 7 => 2,
                6 => 1,
                _ => return None,
            };

            Some(index)
        }))
        .trace("flags2_bytes");

    let body = (
        empty_u8().map_silent(|_| (None, None)), //
        vary_block().map_silent(|vary| (None, Some(vary))),
        fixed_block().map_silent(|fixed| (Some(fixed), None)),
        (fixed_block(), vary_block()).map_silent(|(fixed, vary)| (Some(fixed), Some(vary))),
    )
        .dispatch(kind.output().map(|disc| {
            let index = match disc {
                // Nothing
                0x50 => 0,

                0x07 | 0x20 => 0,
                0x0f | 0x1f | 0x2f | 0x3f | 0x4f | 0x5f | 0x6f | 0x7f | 0x8f | 0xaf | 0xcf
                | 0xdf | 0xef | 0xff => 0,

                0x3b | 0x5b | 0x7b | 0x9b | 0xcb | 0xeb | 0xfb => 0,

                // Vary block
                0x04 | 0x24 | 0x44 | 0x64 | 0x84 | 0xa4 | 0xc4 | 0xe4 | 0xed => 1,
                0x06 | 0x0d | 0x26 | 0x46 | 0x66 | 0x86 | 0xa6 | 0xc6 | 0xe6 => 1,

                // Fixed block
                0x5d => 2,
                0x0b | 0x1b | 0x2b | 0x4b | 0x6b | 0xdb => 2,
                0xec | 0xe0 => 2,
                0x8a | 0x8b | 0x83 => 2,
                0x93 => 2,
                0x94 | 0x97 | 0x9f => 2,
                0x03 | 0x18 | 0x1e | 0x30 | 0x41 => 2,
                0x22 | 0x42 | 0x62 | 0x82 | 0xa2 | 0xc2 | 0xe2 => 2,

                // Fixed + Vary blocks
                0x00 | 0x02 | 0x09 => 3,
                _ => return None,
            };

            Some(index)
        }))
        .trace("body");

    (kind, value, value_v10, bytes, body)
        .map_silent(
            |(kind, value, value_v10, bytes, (fixed_block, vary_block))| Subtile {
                kind,
                value,
                bytes,
                fixed_block,
                vary_block,
            },
        )
        .trace("subtile")
}

fn trailing(
    strings: ForwardRef<String>,
    flags1: ForwardRef<u8>,
    dims: ForwardRef<[u8; 2]>,
    side_gts: ForwardRef<[Option<String>; 4]>,
) -> impl U8Parser<Output = Trailing> {
    let grid_size_1 = dims.map(|[height, width]| (*height as usize + 1) * (*width as usize + 1));

    let flags1_1_gt_grid = indexed_string(strings.clone())
        .repeat_vec(grid_size_1.clone())
        .run_if(side_gts.map(|side_gts| side_gts.iter().any(|gt| gt.is_some())))
        .trace("gt_grid");

    let flags1_0_bools = Bool
        .repeat::<4>()
        .run_if(flags1.clone().map(|f| *f & 1 == 1))
        .trace("flags1[0]");

    let trailing_strings = indexed_string(strings.clone())
        .repeat::<2>()
        .trace("trailing_strings");

    // let flags1_2_bools = Bool
    //     .repeat::<2>()
    //     .run_if(flags1.clone().map(|f| (*f >> 2) & 1 == 1))
    //     .trace("flags1[2]");
    let tail_bools = Bool.repeat::<3>().trace("tail_bools");

    let grid_size_0 = dims.map(|&[h, w]| h as usize * w as usize);

    let flags1_4_blocks = fixed_block()
        .repeat_vec(grid_size_0.clone())
        .run_if(flags1.clone().map(|f| (*f >> 4) & 1 == 1))
        .trace("flags1[4]");

    let flags1_5_floats = f32::LE
        .repeat_vec(grid_size_0.clone())
        .run_if(flags1.clone().map(|f| (*f >> 5) & 1 == 1))
        .trace("flags1[5]");

    let flags1_3_string = indexed_string(strings.clone())
        .run_if(flags1.clone().map(|f| (*f >> 3) & 1 == 1))
        .trace("flags1[3]");

    (
        flags1_1_gt_grid,
        flags1_0_bools,
        trailing_strings,
        // flags1_2_bools,
        tail_bools,
        flags1_3_string,
        flags1_4_blocks,
        flags1_5_floats,
    )
        .map_silent(
            |(
                flags1_1_strings,
                flags1_0_bools,
                trailing_strings,
                // flags1_2_bools,
                tail_bools,
                flags1_3_string,
                flags1_4_blocks,
                flags1_5_floats,
            )| Trailing {
                flags1_1_strings,
                flags1_0_bools,
                strings: trailing_strings,
                tail_bools,
                // flags1_2_bools,
                flags1_3_string: flags1_3_string.flatten(),
                flags1_4_blocks,
                flags1_5_floats,
            },
        )
        .trace("trailing")
}

pub fn tdt_file() -> (impl U8Parser<Output = TDTFile>, ForwardRef<u32>) {
    let version = u32::LE.store().trace("version");
    let version_out = version.output();
    let strings = strings_section().store().trace_opaque("strings_section");

    let header = header(strings.output()).store();

    let tail = tail(strings.output(), version.output()).run_if(header.output().map(|h| {
        h.flags
            .is_none_or(|f| [8, 9, 0xa, 0x1a, 0x18, 0x2a, 0x1c, 0x1e].contains(&f))
    }));

    let parser = (version, strings, header, tail, EoF)
        .map_silent(|(version, strings, header, tail, _)| TDTFile {
            version,
            strings,
            header,
            tail,
        })
        .trace("tdt_file");

    (parser, version_out)
}

pub fn parse_tdt_bytes(mut input: &[u8]) -> Result<TDTFile> {
    let (mut parser, version) = tdt_file();

    let (file, _) = parser
        .parse(&mut input)
        .to_parse_error()
        .with_maybe_version(*version.try_get())?;

    Ok(file)
}
