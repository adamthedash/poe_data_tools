use winnow::{
    Parser,
    ascii::{space0, space1},
    combinator::{opt, preceded, repeat, terminated},
};

use super::types::*;
use crate::file_parsers::{
    error::{AsParseError, ParseResultEx, Result},
    shared::{
        lift::lift,
        winnow::{WinnowParser, filename, quoted, uint, unquoted_str, version_line},
    },
};

fn room<'a>() -> impl WinnowParser<&'a str, Room> {
    winnow::trace!(
        "room",
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
    )
}

pub fn parse_rs_str(contents: &str) -> Result<RSFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let version = lift(version_line())
        .parse_next(&mut lines)
        .to_parse_error()?;

    let mut parser = repeat(0.., lift(room())).map(|rooms| RSFile { version, rooms });

    parser.parse(lines).to_parse_error().with_version(version)
}
