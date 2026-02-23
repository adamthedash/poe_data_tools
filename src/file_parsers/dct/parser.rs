use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, float, space1},
    combinator::{preceded as P, repeat},
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
    )
        .map(|(weight, atlas_file, tag, float)| Entry {
            weight,
            atlas_file,
            tag,
            float,
        })
        .trace("entry")
}

fn group<'a>() -> impl SliceParser<'a, &'a str, Group> {
    (
        lift(quoted_str), //
        repeat(0.., lift(entry())),
    )
        .map(|(area, entries)| Group { area, entries })
        .trace("group")
}

fn default_group<'a>() -> impl SliceParser<'a, &'a str, Group> {
    (
        lift(literal("Default").map(String::from)), //
        repeat(0.., lift(entry())),
    )
        .map(|(area, entries)| Group { area, entries })
        .trace("default_group")
}

pub fn parse_dct_str(contents: &str) -> Result<DCTFile> {
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
