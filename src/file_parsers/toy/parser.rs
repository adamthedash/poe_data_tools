use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, space1},
    combinator::{alt, opt, preceded as P, repeat, separated_pair},
    token::{literal, take_while},
};

use super::types::*;
use crate::file_parsers::{
    lift::{SliceParser, lift},
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
    let primary_stuff = (
        dec_uint, //
        P(space1, quoted('"').and_then(filename("arm"))),
    );

    enum Tail {
        KV(String, String),
        NotFlag(String),
        AddFlag(String),
        Rotation(Rotation),
    }

    // Tail stuff can be in any order, so collect them with an enum then distribute them into
    // their destinations
    let tail_stuff = repeat(
        0..,
        P(
            space1,
            alt((
                P("!", unquoted_str).map(Tail::NotFlag),
                d4_rotation().map(Tail::Rotation),
                flag().map(Tail::AddFlag),
                separated_pair(
                    take_while(1.., |c: char| c != '=' && !c.is_whitespace()).map(String::from),
                    "=",
                    unquoted_str,
                )
                .map(|(k, v)| Tail::KV(k, v)),
            )),
        ),
    )
    .fold(
        || (vec![], vec![], vec![], vec![]),
        |(mut kvs, mut not_flags, mut add_flags, mut rotations), item| {
            match item {
                Tail::KV(k, v) => kvs.push((k, v)),
                Tail::NotFlag(f) => not_flags.push(f),
                Tail::Rotation(r) => rotations.push(r),
                Tail::AddFlag(f) => add_flags.push(f),
            };

            (kvs, not_flags, add_flags, rotations)
        },
    );

    (
        primary_stuff, //
        tail_stuff,
    )
        .map(
            |((weight, arm_file), (key_values, not_flags, add_flags, rotations))| Entry {
                weight,
                arm_file,
                key_values,
                not_flags,
                add_flags,
                rotations,
            },
        )
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
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let version = lift(version_line())
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = repeat(0.., group()).map(|groups| TOYFile { version, groups });

    let file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(file)
}
