use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint as U, float as F, space1},
    combinator::{alt, opt, preceded as P, repeat},
    token::literal,
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::{SliceParser, lift},
    shared::winnow::{
        TraceHelper, WinnowParser, filename, quoted, quoted_str, safe_u32, unquoted_str,
        version_line,
    },
};

fn line1<'a>() -> impl WinnowParser<&'a str, Line1> {
    (
        F, //
        opt(P(space1, U)),
        opt(P(space1, U)),
    )
        .map(|(scale, uint1, uint2)| Line1 {
            scale,
            uint1,
            uint2,
        })
        .trace("line1")
}

fn group_header<'a>(
    name_parser: impl WinnowParser<&'a str, String>,
) -> impl WinnowParser<&'a str, (String, Option<String>, Option<f32>)> {
    (
        name_parser, //
        opt(P(space1, unquoted_str)),
        opt(P(space1, F)),
    )
        .trace("group_header")
}

fn object<'a>() -> impl WinnowParser<&'a str, Object> {
    (
        alt((literal("All").map(|_| Weight::All), F.map(Weight::Float))),
        P(space1, quoted('"').and_then(filename("ao"))),
        opt(P(space1, safe_u32)),
        opt(P(space1, literal("D").map(String::from))),
        opt(P(space1, F)),
    )
        .map(|(weight, ao_file, uint1, d, float1)| Object {
            weight,
            ao_file,
            uint1,
            d,
            float1,
        })
        .trace("object")
}

fn group<'a>(
    name_parser: impl WinnowParser<&'a str, String>,
) -> impl SliceParser<'a, &'a str, Group> {
    (
        lift(group_header(name_parser)), //
        repeat(0.., lift(object())),
    )
        .map(|((name, d, float1), objects)| Group {
            name,
            d,
            float1,
            objects,
        })
        .trace("group")
}

pub fn parse_ddt_str(contents: &str) -> Result<DDTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();

    let mut parser = (
        lift(version_line()),
        lift(line1()),
        opt(lift(U)),
        group(unquoted_str),
        repeat(0.., group(quoted_str)),
    )
        .map(|(version, line1, uint1, default_group, mut groups)| {
            Vec::insert(&mut groups, 0, default_group); // for type inference
            DDTFile {
                version,
                line1,
                uint1,
                groups,
            }
        });

    let ddt_file = parser
        .parse(&lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(ddt_file)
}
