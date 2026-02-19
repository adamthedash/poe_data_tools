use std::collections::HashMap;

use winnow::{
    Parser,
    binary::{le_u16, le_u32, length_repeat, u8 as U8},
    combinator::{alt, peek, repeat},
    token::{rest, take},
};

use super::types::TDTFile;
use crate::file_parsers::shared::winnow::WinnowParser;

fn strings_section<'a>() -> impl WinnowParser<&'a [u8], Vec<String>> {
    winnow::trace!(
        "string_section",
        length_repeat(le_u32, le_u16).map(|tokens: Vec<_>| {
            tokens
                .split(|c| *c == 0)
                .map(|chars| String::from_utf16(chars).expect("Invalid utf16 char"))
                .collect::<Vec<_>>()
        })
    )
}

pub fn parse_tdt_bytes(mut contents: &[u8]) -> winnow::Result<TDTFile> {
    let (version, strings) = (
        le_u32, //
        strings_section(),
    )
        .parse_next(&mut contents)?;

    let num1 = le_u32(&mut contents)?;

    let peeked = peek(U8).parse_next(&mut contents)?;
    if peeked > 0 {
        // advance
        take(1_usize).parse_next(&mut contents)?;
    } else {
        // Don't advance, throw out value
        take(4_usize).parse_next(&mut contents)?;
    }

    let num2 = le_u32(&mut contents)?;

    let nums1 = vec![num1, peeked as u32, num2];

    let nums3: Vec<_> =
        repeat(4, le_u32.map(|x| (x < u32::MAX).then_some(x))).parse_next(&mut contents)?;

    let rest = alt((
        take(128_usize), //
        rest,
    ))
    .parse_next(&mut contents)?;

    let (_, string_offsets) = strings
        .iter()
        .fold((0, vec![]), |(mut offset, mut offsets), s| {
            offsets.push(offset);
            offset += s.len() + 1;

            (offset, offsets)
        });

    let string_lut = string_offsets
        .iter()
        .copied()
        .zip(strings.iter().cloned())
        .collect::<HashMap<_, _>>();

    let nums3_string = nums3
        .iter()
        // .map(|o| o.and_then(|o| string_lut.get(&(o as usize)).cloned()))
        .map(|o| o.map(|o| string_lut[&(o as usize)].clone()))
        .collect();

    let tdt_file = TDTFile {
        version,
        strings,
        strings_offsets: string_offsets,
        nums1,
        // num2,
        nums3,
        nums3_string,
        // nums4,
        rest: rest.to_vec(),
    };

    Ok(tdt_file)
}
