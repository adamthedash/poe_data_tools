use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, float, space1},
    combinator::{alt, cond, opt, preceded as P, repeat},
};

use super::types::*;
use crate::file_parsers::shared::{
    lift::{SliceParser, lift},
    winnow::{TraceHelper, WinnowParser, filename, quoted, quoted_str, unquoted_str, version_line},
};

pub fn item<'a>(version: u32) -> impl WinnowParser<&'a str, Item> {
    (
        dec_uint,
        P(space1, quoted_str),
        cond(
            version >= 4,
            P(space1, quoted('"').and_then(filename("ao"))),
        ),
        P(space1, float),
        cond(
            version >= 3, //
            P(space1, float),
        ),
        P(space1, dec_uint),
        P(space1, dec_uint),
    )
        .map(
            |(uint1, stub, ao_file, float1, float2, uint2, uint3)| Item {
                uint1,
                stub,
                ao_file,
                float1,
                float2,
                uint2,
                uint3,
            },
        )
        .trace("item")
}

pub fn group<'a>(version: u32) -> impl SliceParser<'a, &'a str, Group> {
    (
        lift((
            alt((quoted_str, unquoted_str)), //
            opt(P(space1, float)),
        )),
        repeat(0.., lift(item(version))),
    )
        .map(|((name, float), items)| Group { name, float, items })
        .trace("group")
}

pub fn parse_clt_str(contents: &str) -> Result<CLTFile> {
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
        lift(float), //
        repeat(1.., group(version)),
    )
        .map(|(float1, groups)| CLTFile {
            version,
            float1,
            groups,
        });

    let clt_file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(clt_file)
}
