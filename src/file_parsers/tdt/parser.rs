use std::{collections::HashMap, fmt::Display};

use winnow::{
    Parser,
    binary::{le_u8, le_u16, le_u32, length_repeat, length_take},
    combinator::{cond, dispatch, empty, fail, repeat, terminated},
    error::ContextError,
    token::{any, rest, take, take_till},
};

use super::types::*;
use crate::file_parsers::shared::winnow::{WinnowParser, repeat_array};

#[derive(Debug)]
enum Error {
    IndexOutOfBounds { index: usize },
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IndexOutOfBounds { .. } => f.write_str("Index out of bounds"),
        }
    }
}

fn utf16_null_terminated<'a>() -> impl WinnowParser<&'a [u16], String> {
    winnow::trace!(
        "utf16_null_terminated",
        terminated(take_till(0.., |c| c == 0), any.verify(|c| *c == 0)) //
            .try_map(String::from_utf16)
    )
}

fn strings_section<'a>() -> impl WinnowParser<&'a [u8], HashMap<usize, Option<String>>> {
    let parser = length_repeat(le_u32, le_u16)
        .try_map(|chars: Vec<_>| {
            repeat(1.., utf16_null_terminated())
                .parse(&chars)
                .map_err(|e| e.into_inner())
        })
        .map(|strings: Vec<_>| {
            strings
                .into_iter()
                .scan(0, |offset, s| {
                    let key_string = *offset;
                    *offset += s.len();

                    // Also include lookup location for empty strings
                    let key_null = *offset;
                    *offset += 1;

                    let key_vals = [(key_string, Some(s)), (key_null, None)];

                    Some(key_vals)
                })
                .flatten()
                .collect()
        });

    winnow::trace!("string_section", parser)
}

fn string<'a>(
    strings: &HashMap<usize, Option<String>>,
) -> impl WinnowParser<&'a [u8], Option<String>> {
    winnow::trace!(
        "string",
        le_u32.try_map(|i| {
            let Some(string) = strings.get(&(i as usize)) else {
                return Err(Error::IndexOutOfBounds { index: i as usize });
            };

            Ok(string.clone())
        })
    )
}

fn opt_string<'a>(
    strings: &HashMap<usize, Option<String>>,
) -> impl WinnowParser<&'a [u8], Option<String>> {
    let parser = le_u32.try_map(|i| {
        if i == u32::MAX {
            return Ok(None);
        }

        // TODO: I don't think optional strings ever point to a null char, but need to verify

        let Some(string) = strings.get(&(i as usize)) else {
            return Err(Error::IndexOutOfBounds { index: i as usize });
        };

        Ok(string.clone())
    });

    winnow::trace!("opt_string", parser)
}

fn header<'a>(
    version: u32,
    strings: &HashMap<usize, Option<String>>,
) -> impl WinnowParser<&'a [u8], Header> {
    // Inconsistent part
    let files = move |input: &mut &[u8]| -> winnow::Result<_> {
        let tdt_file = string(strings).parse_next(input)?;

        let (flags, num1, tgt_tmd_file, tag) = if let Some(tdt_file) = &tdt_file {
            assert!(tdt_file.ends_with(".tdt"));

            let flags = any(input)?;
            let mut parser = (
                cond([1, 2].contains(&flags), le_u16), //
                cond([1, 2, 9].contains(&flags), string(strings)),
                cond(
                    [8, 9, 0xa, 0x1a, 0x18, 0x2a, 0x1c, 0x1e].contains(&flags),
                    string(strings),
                ),
            );

            let (num1, tgt_tmd_file, tag) = parser.parse_next(input)?;

            (Some(flags), num1, tgt_tmd_file.flatten(), tag.flatten())
        } else {
            let tgt_file = string(strings)
                .parse_next(input)?
                .expect("tgt file should not be empty");
            let tag = string(strings).parse_next(input)?;

            (None, None, Some(tgt_file), tag)
        };

        Ok((tdt_file, flags, num1, tgt_tmd_file, tag))
    };

    let parser = (
        files,
        repeat_array(opt_string(strings)),
        dispatch! {
            empty.value(version);
            ..4 => repeat_array(le_u32.map(|x| { assert!(x < 256); x as u8 })),
            4.. => repeat_array(le_u8),
        },
        repeat_array(opt_string(strings)),
        cond(version >= 4, repeat_array(le_u8)),
        repeat_array(le_u8),
        dispatch! {
            empty.value(version);
            ..4 => repeat_array(le_u32.map(|x| { assert!(x < 256); x as u8 })),
            4.. => repeat_array(le_u8),
        },
        repeat_array(le_u8),
    )
        .map(
            |(
                (tdt_file, flags, num1, tgt_tmd_file, tag),
                side_ets,
                dimensions,
                side_gts,
                dimensions2,
                extra_nums,
                side_offsets,
                [flags1, flags2],
            )| {
                Header {
                    tdt_file,
                    flags,
                    num1,
                    tgt_tmd_file,
                    tag,
                    side_ets,
                    dimensions,
                    side_gts,
                    dimensions2,
                    extra_nums,
                    side_offsets,
                    flags1,
                    flags2,
                }
            },
        );

    winnow::trace!("header", parser)
}

fn subtile<'a>(
    _version: u32,
    _strings: &HashMap<usize, Option<String>>,
) -> impl WinnowParser<&'a [u8], Subtile> {
    let parser = move |input: &mut &[u8]| {
        let (kind, value, string) = (le_u8, le_u8, le_u8).parse_next(input)?;

        let fixed_block = dispatch! {
            empty.value(kind);
            7 | 15 | 31 | 224 | 226 | 228 | 230 | 237 => empty.value(None),
            3 | 33 => take(1_usize).map(Vec::from).map(Some),
            _ => fail,
        }
        .parse_next(input)?;

        let variable_block = dispatch! {
            empty.value(kind);
            3 | 7 | 15 | 31 | 33 => empty.value(None),
            224 | 226 | 228 | 230 | 237=> length_take(le_u16).map(Vec::from).map(Some),
            _ => fail,
        }
        .parse_next(input)?;

        Ok(Subtile {
            kind,
            value,
            fixed_block,
            variable_block,
            // string,
            string_index: string,
        })
    };

    winnow::trace!("subtile", parser)
}

fn tdt_file<'a>() -> impl WinnowParser<&'a [u8], TDTFile> {
    let parser = |input: &mut &[u8]| {
        let (version, strings) = (
            le_u32, //
            strings_section(),
        )
            .parse_next(input)?;

        let header = header(version, &strings).parse_next(input)?;

        // let flags = le_u8
        //     .map(|b: u8| array::from_fn(|i| (b >> i) & 1))
        //     .parse_next(input)?;
        // let bytes = repeat_array(le_u8).parse_next(input)?;

        // let num_subtiles = le_u16(input)?;

        // let mut subtiles = vec![];
        // for _ in 0..num_subtiles {
        //     let checkpoint = input.checkpoint();
        //     let Ok(subtile) = subtile(version, &strings).parse_next(input) else {
        //         input.reset(&checkpoint);
        //         break;
        //     };
        //
        //     subtiles.push(subtile);
        // }

        let rest = rest::<_, ContextError>.parse_next(input)?;

        let dims = header.dimensions.iter().map(|x| *x as u16).product::<u16>();
        let offset = rest
            .windows(2)
            .take(10)
            .position(|w| le_u16::<_, ContextError>.parse(w).unwrap() == dims)
            .unwrap_or(usize::MAX);

        // // parse from end
        // let trailing_uint1 = if flags[5] == 1 {
        //     let trailing_uint1 = rest[rest.len() - 1];
        //     rest = &rest[..rest.len() - 1];
        //     Some(trailing_uint1)
        // } else {
        //     None
        // };
        //
        // let trailing_uint2 = if flags[4] == 1 {
        //     let trailing_uint2 = rest[rest.len() - 1];
        //     rest = &rest[..rest.len() - 1];
        //     Some(trailing_uint2)
        // } else {
        //     None
        // };
        //
        // let input1 = rest
        //     .get(rest.len() - (4 * 2 + 1 + 1)..)
        //     .ok_or_else(ContextError::new)?;
        // let (dims3, trailing_uint3, trailing_uint4) = (
        //     repeat_array(repeat_array(le_u8)), //
        //     le_u8,
        //     le_u8,
        // )
        //     .parse(input1)
        //     .map_err(|e| e.into_inner())?;
        //
        // rest = &rest[..rest.len() - (4 * 2 + 1 + 1)];

        let tdt_file = TDTFile {
            version,
            strings,
            header,
            num_tiles_offset: offset,
            // flags,
            // bytes,
            // num_subtiles,
            // subtiles,
            rest: rest.to_vec(),
            // dims3,
            // trailing_uint3,
            // trailing_uint4,
            // trailing_uint2,
            // trailing_uint1,
        };

        Ok(tdt_file)
    };

    winnow::trace!("tdt_file", parser)
}

pub fn parse_tdt_bytes(contents: &[u8]) -> winnow::Result<TDTFile> {
    tdt_file().parse(contents).map_err(|e| e.into_inner())
}
