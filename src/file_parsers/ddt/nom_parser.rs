use anyhow::{Result, anyhow};
use nom::{
    Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{space1, u32 as U},
    combinator::{all_consuming, opt},
    multi::many0,
    number::complete::float as F,
    sequence::preceded as P,
};

use super::types::*;
use crate::file_parsers::{
    lift_nom::{SliceParser, ToSliceParser},
    shared::{NomParser, quoted_str, safe_u32, unquoted_str, version_line},
    slice::Slice,
};

fn line1<'a>() -> impl NomParser<'a, Line1> {
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
}

fn group_header<'a>(
    name_parser: impl NomParser<'a, String>,
) -> impl NomParser<'a, (String, Option<String>, Option<f32>)> {
    (
        name_parser, //
        opt(P(space1, unquoted_str)),
        opt(P(space1, F)),
    )
}

fn object<'a>() -> impl NomParser<'a, Object> {
    (
        alt((tag("All").map(|_| Weight::All), F.map(Weight::Float))),
        P(space1, quoted_str),
        opt(P(space1, safe_u32)),
        opt(P(space1, tag("D").map(String::from))),
        opt(P(space1, F)),
    )
        .map(|(weight, ao_file, uint1, d, float1)| Object {
            weight,
            ao_file,
            uint1,
            d,
            float1,
        })
}

fn group<'a>(name_parser: impl NomParser<'a, String>) -> impl SliceParser<'a, &'a str, Group> {
    (
        group_header(name_parser).lift(), //
        many0(object().lift()),
    )
        .map(|((name, d, float1), objects)| Group {
            name,
            d,
            float1,
            objects,
        })
}

pub fn parse_ddt_str(contents: &str) -> Result<DDTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let lines = Slice(lines.as_slice());

    let parser = (
        version_line().lift(),
        line1().lift(),
        opt(U::<_, nom::error::Error<_>>.lift()),
        group(unquoted_str),
        many0(group(quoted_str)),
    )
        .map(|(version, line1, uint1, default_group, mut groups)| {
            groups.insert(0, default_group);
            DDTFile {
                version,
                line1,
                uint1,
                groups,
            }
        });

    let (_, ddt_file) = all_consuming(parser)
        .parse_complete(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(ddt_file)
}
