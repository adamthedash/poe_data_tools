use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, float, space1},
    combinator::{alt, delimited, opt, preceded as P, repeat, separated_pair},
    token::literal,
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::lift,
    shared::winnow::{TraceHelper, WinnowParser, filename, parse_bool, quoted, version_line},
};

fn num_line<'a>() -> impl WinnowParser<&'a str, Nums> {
    (
        float, //
        P(space1, float),
        P(space1, parse_bool),
        P(space1, parse_bool),
        opt(P(space1, dec_uint)),
    )
        .map(|(float1, float2, bool1, bool2, uint)| Nums {
            float1,
            float2,
            bool1,
            bool2,
            uint,
        })
        .trace("num_line")
}

fn entry<'a>() -> impl WinnowParser<&'a str, Entry> {
    (
        quoted('"').and_then(filename("fmt")), //
        P(space1, float),
        repeat(
            0..,
            P(
                space1,
                alt((
                    delimited(
                        literal('('),
                        separated_pair(float, literal(','), float),
                        literal(')'),
                    ),
                    delimited(
                        literal('['),
                        separated_pair(float, space1, float),
                        literal(']'),
                    ),
                )),
            ),
        ),
    )
        .map(|(fmt_file, float, points)| Entry {
            fmt_file,
            float,
            points,
        })
        .trace("entry")
}

pub fn parse_dlp_str(contents: &str) -> Result<DLPFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let mut parser = (
        opt(lift(version_line())),
        lift(num_line()), //
        repeat(0.., lift(entry())),
    )
        .map(|(version, nums, entries)| DLPFile {
            version,
            nums,
            entries,
        });

    let pet_file = parser
        .parse(lines.as_slice())
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(pet_file)
}
