use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, space1},
    combinator::{alt, opt, preceded as P, repeat},
    token::literal,
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::{SliceParser, lift},
    shared::winnow::{
        TraceHelper, WinnowParser, filename, parse_bool, quoted, unquoted, unquoted_str,
        version_line,
    },
};

/// One of: [I, R90, R180, R270, FI, FR90, FR180, FR270]
fn d4_rotation<'a>() -> impl WinnowParser<&'a str, Rotation> {
    (
        opt(literal('F')).map(|f| f.is_some()),
        alt((
            literal('I').value(0), //
            P(literal('R'), dec_uint),
        )),
    )
        .map(|(flip, angle)| Rotation { flip, angle })
        .trace("d4_rotation")
}

/// +/- followed by a flag name
fn flag<'a>() -> impl WinnowParser<&'a str, String> {
    (
        alt((
            literal('+'), //
            literal('-'),
        )),
        unquoted(),
    )
        .map(|(prefix, name)| [prefix, name].concat())
        .trace("flag")
}

fn header<'a>() -> impl WinnowParser<&'a str, Header> {
    (
        parse_bool,
        P(space1, parse_bool),
        opt(P(
            space1,
            alt((
                literal("FileOrder").value(Order::File),
                literal("SizeOrder").value(Order::Size),
            )),
        )),
        repeat(0.., P(space1, flag())),
    )
        .map(|(uint1, uint2, file_order, flags)| Header {
            bool1: uint1,
            bool2: uint2,
            file_order,
            flags,
        })
        .trace("header")
}

fn entry<'a>() -> impl WinnowParser<&'a str, Entry> {
    (
        dec_uint, //
        P(space1, quoted('"').and_then(filename("arm"))),
        opt(P((space1, literal("limit=")), dec_uint)),
        repeat(0.., P((space1, literal('!')), unquoted_str)),
        repeat(0..=8, P(space1, d4_rotation())),
    )
        .map(|(weight, arm_file, limit, flags, rotations)| Entry {
            weight,
            arm_file,
            limit,
            flags,
            rotations,
        })
        .trace("entry")
}

fn group<'a>() -> impl SliceParser<'a, &'a str, Group> {
    (
        lift(header()), //
        repeat(0.., lift(entry())),
    )
        .map(|(header, entries)| Group { header, entries })
        .trace("group")
}

pub fn parse_toy_str(contents: &str) -> Result<TOYFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("#"))
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let version = lift(version_line())
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = repeat(1.., group()).map(|groups| TOYFile { version, groups });

    let file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(file)
}
