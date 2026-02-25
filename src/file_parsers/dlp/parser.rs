use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, float, space1},
    combinator::{
        alt, delimited, dispatch, empty, eof, opt, preceded as P, repeat, separated_pair,
        terminated,
    },
    token::{literal, rest},
};

use super::types::*;
use crate::file_parsers::{
    lift::{SliceParser, lift},
    shared::winnow::{
        TraceHelper, WinnowParser, filename, parse_bool, quoted, unquoted, version_line,
    },
};

fn headers_v2<'a>() -> impl WinnowParser<&'a str, HeadersV2> {
    (
        float, //
        P(space1, float),
        P(space1, parse_bool),
        P(space1, parse_bool),
        opt(P(space1, dec_uint)),
        opt(P(space1, dec_uint)),
        opt(P(space1, dec_uint)),
        opt(P(space1, float)),
        opt(P(space1, dec_uint)),
        opt(P(space1, float)),
    )
        .map(
            |(
                scale_min,
                scale_max,
                allow_waving,
                allow_on_blocking,
                max_rotation,
                uint1,
                uint2,
                float1,
                audio_type,
                float2,
            )| HeadersV2 {
                scale_min,
                scale_max,
                allow_waving,
                allow_on_blocking,
                max_rotation,
                uint1,
                uint2,
                float1,
                audio_type,
                float2,
            },
        )
        .trace("headers_v2")
}

fn headers_v3<'a>() -> impl SliceParser<'a, &'a str, HeadersV3> {
    repeat(
        0..,
        lift(P(
            (literal('-'), space1),
            dispatch! {
                terminated(unquoted(), opt(space1));
                "RandomScale" => separated_pair(float, space1, float).map(|(min, max)| Header::RandomScale { min, max }),
                "AllowWaving" => eof.value(Header::AllowWaving),
                "AllowOnBlocking" => eof.value(Header::AllowOnBlocking),
                "MaxRotation" => dec_uint.map(Header::MaxRotation),
                "MinEdgeScale" => float.map(Header::MinEdgeScale),
                "AudioType" => dec_uint.map(Header::AudioType),
                "DelayMultiplier" => float.map(Header::DelayMultiplier),
                "TimeMultiplier" => float.map(Header::TimeMultiplier),
                "SizeMultiplier" => float.map(Header::SizeMultiplier),
                "Seed" => dec_uint.map(Header::Seed),
                key => rest.map(move |rest: &str| Header::Other { key: key.to_string(), rest: rest.to_string() }),
            },
        )),
    )
    .map(HeadersV3)
    .trace("headers_v3")
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
    let mut lines = lines.as_slice();

    let version = opt(lift(version_line()))
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = (
        dispatch! {
            empty.value(version.unwrap_or(0));
            ..3 => lift(headers_v2()).map(Headers::V2),
            3.. => headers_v3().map(Headers::V3),
        },
        repeat(0.., lift(entry())),
    )
        .map(|(headers, entries)| DLPFile {
            version,
            headers,
            entries,
        });

    let pet_file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(pet_file)
}
