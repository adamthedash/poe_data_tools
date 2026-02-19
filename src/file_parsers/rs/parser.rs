use anyhow::{Result, anyhow};
use nom::{
    Parser,
    character::complete::{space0, space1, u32 as U},
    combinator::{all_consuming, opt},
    multi::many0,
    sequence::{preceded, terminated},
};

use super::types::*;
use crate::file_parsers::{
    lift::ToSliceParser,
    shared::{NomParser, quoted_str, unquoted_str, version_line},
    slice::Slice,
};

fn room<'a>() -> impl NomParser<'a, Room> {
    (
        opt(terminated(U, space0)),
        quoted_str,
        all_consuming(many0(preceded(space1, unquoted_str))),
    )
        .map(|(weight, arm_file, rotations)| Room {
            weight,
            arm_file,
            rotations,
        })
}

pub fn parse_rs_str(contents: &str) -> Result<RSFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        // Skip empty/commented lines
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let lines = Slice(lines.as_slice());

    let parser = (
        version_line().lift(), //
        many0(room().lift()),
    )
        .map(|(version, rooms)| RSFile { version, rooms });

    let (_, room_file) = all_consuming(parser)
        .parse_complete(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(room_file)
}
