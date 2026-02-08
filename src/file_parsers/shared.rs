use anyhow::{Context, bail};
use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_till1, take_until},
    character::complete::{char as C, u32 as U},
    sequence::{delimited, preceded as P},
};

use super::line_parser::{MultilineParser, nom_adapter, single_line};

/// Parses a 0/1 as a bool
pub fn parse_bool(line: &str) -> IResult<&str, bool> {
    let (rest, item) = U(line)?;

    let item = match item {
        0 => false,
        1 => true,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                line,
                // No better option here
                nom::error::ErrorKind::Digit,
            )));
        }
    };

    Ok((rest, item))
}

pub fn quoted_str(input: &str) -> IResult<&str, String> {
    delimited(C('"'), take_until("\""), C('"'))
        .map(String::from)
        .parse(input)
}

pub fn unquoted_str(input: &str) -> IResult<&str, String> {
    take_till1(|c: char| c.is_whitespace())
        .map(String::from)
        .parse(input)
}

pub fn version_line<'a>() -> impl MultilineParser<'a, u32> {
    single_line(nom_adapter(P(tag("version "), U)))
}

/// Parse the bytes of a UTF-16 file with BOM
/// https://en.wikipedia.org/wiki/Byte_order_mark#UTF-16
pub fn utf16_bom_to_string(contents: &[u8]) -> anyhow::Result<String> {
    let parse_ut16 = match &contents[..2] {
        [0xff, 0xfe] => String::from_utf16le,
        [0xfe, 0xff] => String::from_utf16be,
        bytes => bail!("Invalid BOM found: {:?}", bytes),
    };

    parse_ut16(&contents[2..]).context("Failed to parse contents as UTF-16 string")
}
