use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, float, space1},
    combinator::{cond, preceded as P, repeat, separated},
    token::{literal, take_while},
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::{SliceParser, lift},
    shared::winnow::{TraceHelper, WinnowParser, quoted, version_line},
};

/// "hello, world"
fn quoted_comma_separated<'a>() -> impl WinnowParser<&'a str, Vec<String>> {
    quoted('"')
        .and_then(separated(
            1..,
            take_while(1.., |c| c != ',').map(String::from),
            literal(", "),
        ))
        .trace("quoted_comma_separated")
}

fn num_line<'a>(version: u32) -> impl WinnowParser<&'a str, Nums> {
    (
        float, //
        P(space1, float),
        P(space1, dec_uint),
        P(space1, dec_uint),
        cond(version >= 3, P(space1, dec_uint)),
    )
        .map(|(float1, float2, uint1, uint2, uint3)| Nums {
            float1,
            float2,
            uint1,
            uint2,
            uint3,
        })
        .trace("num_line")
}

fn entry<'a>() -> impl WinnowParser<&'a str, Entry> {
    (
        dec_uint, //
        P(space1, quoted_comma_separated()),
    )
        .map(|(weight, chest_types)| Entry {
            weight,
            chest_types,
        })
        .trace("entry")
}

fn group<'a>(named: bool) -> impl SliceParser<'a, &'a str, Group> {
    (
        cond(named, lift(quoted_comma_separated())), //
        repeat(0.., lift(entry())),
    )
        .map(|(areas, entries)| Group {
            areas: areas.unwrap_or(vec!["Default".to_string()]),
            entries,
        })
        .trace("group")
}

pub fn parse_cht_str(contents: &str) -> Result<CHTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("#"))
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let version = lift(version_line())
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = (
        lift(num_line(version)), //
        group(false).trace("default_group"),
        repeat(0.., group(true)),
    )
        .map(|(nums, default_group, mut groups)| {
            Vec::insert(&mut groups, 0, default_group);
            CHTFile {
                version,
                nums,
                groups,
            }
        });

    let pet_file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(pet_file)
}
