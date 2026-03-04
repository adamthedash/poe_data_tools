use std::{collections::HashMap, fmt::Display};

use winnow::{
    Parser,
    binary::{le_u8, le_u16, le_u32, length_repeat},
    combinator::{cond, dispatch, empty, repeat, terminated},
    error::ContextError,
    token::{any, rest, take_till},
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
    let parser = (
        cond(version >= 7, string(strings)),
        cond(version >= 5, string(strings)),
        string(strings),
        repeat_array(opt_string(strings)),
        dispatch! {
            empty.value(version);
            ..4 => repeat_array(le_u32.map(|x| { assert!(x < 256); x as u8 })),
            4.. => repeat_array(le_u8),
        },
        repeat_array(opt_string(strings)),
        cond(version >= 4, repeat_array(le_u8)),
        dispatch! {
            empty.value(version);
            ..4 => repeat_array(le_u32.map(|x| { assert!(x < 256); x as u8 })),
            4.. => repeat_array(le_u8),
        },
    )
        .map(
            |(
                unk_string,
                common_tgt,
                tag,
                side_ets,
                dimensions,
                side_gts,
                dimensions2,
                side_offsets,
            )| {
                Header {
                    unk_string: unk_string.flatten(),
                    common_tgt: common_tgt.flatten(),
                    tag,
                    side_ets,
                    dimensions,
                    side_gts,
                    dimensions2,
                    side_offsets,
                }
            },
        );

    winnow::trace!("header", parser)
}

fn tdt_file<'a>() -> impl WinnowParser<&'a [u8], TDTFile> {
    let parser = |contents: &mut &[u8]| {
        let (version, strings) = (
            le_u32, //
            strings_section(),
        )
            .parse_next(contents)?;

        println!("{:#?}", strings);

        let header = header(version, &strings).parse_next(contents)?;

        let rest = rest::<_, ContextError>.parse_next(contents)?;

        let tdt_file = TDTFile {
            version,
            strings,
            header,
            rest: rest.to_vec(),
        };

        Ok(tdt_file)
    };

    winnow::trace!("tdt_file", parser)
}

pub fn parse_tdt_bytes(contents: &[u8]) -> winnow::Result<TDTFile> {
    tdt_file().parse(contents).map_err(|e| e.into_inner())
}
