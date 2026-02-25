use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, float, space1},
    combinator::{alt, cond, dispatch, empty, opt, preceded as P, repeat},
};

use super::types::*;
use crate::file_parsers::shared::{
    lift::{SliceParser, lift},
    winnow::{TraceHelper, WinnowParser, quoted_comma_separated, version_line},
};

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

fn group<'a, const NAMED: bool>(version: u32) -> impl SliceParser<'a, &'a str, Group> {
    let header = dispatch! {
        empty.value(NAMED);
        false => num_line(version).map(|nums| (vec!["Default".to_string()], Some(NumLine::V3(nums)))),
        true => (
                    quoted_comma_separated(),
                    opt(P(
                        space1,
                        alt((
                            num_line(version).map(NumLine::V3), //
                            float.map(NumLine::V2),
                        )),
                    )),
                ),
    };

    (
        lift(header), //
        repeat(0.., lift(entry())),
    )
        .map(|((areas, nums), entries)| Group {
            areas,
            entries,
            nums,
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
        group::<false>(version).trace("default_group"),
        repeat(0.., group::<true>(version)),
    )
        .map(|(default_group, mut groups)| {
            Vec::insert(&mut groups, 0, default_group);
            CHTFile { version, groups }
        });

    let pet_file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(pet_file)
}
