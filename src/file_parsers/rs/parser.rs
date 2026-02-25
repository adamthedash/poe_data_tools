use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{space0, space1},
    combinator::{opt, preceded, repeat, terminated},
};

use super::types::*;
use crate::file_parsers::{
    lift::lift,
    shared::winnow::{
        TraceHelper, WinnowParser, filename, quoted, uint, unquoted_str, version_line,
    },
};

fn room<'a>() -> impl WinnowParser<&'a str, Room> {
    (
        opt(terminated(uint, space0)),
        quoted('"').and_then(filename("arm")),
        repeat(0.., preceded(space1, unquoted_str)),
    )
        .map(|(weight, arm_file, rotations)| Room {
            weight,
            arm_file,
            rotations,
        })
        .trace("room")
}

pub fn parse_rs_str(contents: &str) -> Result<RSFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();

    let mut parser = (
        lift(version_line()), //
        repeat(0.., lift(room())),
    )
        .map(|(version, rooms)| RSFile { version, rooms });

    let room_file = parser
        .parse(lines.as_slice())
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(room_file)
}
