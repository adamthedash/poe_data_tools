use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, float, space1},
    combinator::{opt, preceded as P, repeat},
    token::literal,
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::{SliceParser, lift},
    shared::winnow::{TraceHelper, WinnowParser, filename, quoted, quoted_str, version_line},
};

fn entry<'a>() -> impl WinnowParser<&'a str, Entry> {
    (
        dec_uint, //
        P(space1, quoted('"').and_then(filename("atlas"))),
        P(space1, quoted_str),
        P(space1, float),
        opt(P(space1, float)),
    )
        .map(|(weight, atlas_file, tag, float1, float2)| Entry {
            weight,
            atlas_file,
            tag,
            float1,
            float2,
        })
        .trace("entry")
}

fn header<'a>() -> impl WinnowParser<&'a str, (String, Option<f32>)> {
    (
        quoted_str, //
        opt(P(space1, float)),
    )
}

fn group<'a>() -> impl SliceParser<'a, &'a str, Group> {
    (
        lift(header()), //
        repeat(0.., lift(entry())),
    )
        .map(|((area, float), entries)| Group {
            area,
            float,
            entries,
        })
        .trace("group")
}

fn default_group<'a>() -> impl SliceParser<'a, &'a str, Group> {
    (
        lift(literal("Default").map(String::from)), //
        repeat(0.., lift(entry())),
    )
        .map(|(area, entries)| Group {
            area,
            float: None,
            entries,
        })
        .trace("default_group")
}

pub fn parse_dct_str(contents: &str) -> Result<DCTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let version = lift(version_line())
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = (
        lift(float),
        default_group(), //
        repeat(0.., group()),
    )
        .map(|(float, default_group, mut groups)| {
            Vec::insert(&mut groups, 0, default_group);
            DCTFile {
                version,
                float,
                groups,
            }
        });

    let pet_file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(pet_file)
}
