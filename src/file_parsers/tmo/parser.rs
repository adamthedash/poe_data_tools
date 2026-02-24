use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::space1,
    combinator::{repeat, separated_pair},
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::lift,
    shared::winnow::{filename, quoted, version_line},
};

pub fn parse_tmo_str(contents: &str) -> Result<TMOFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let version = lift(version_line())
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = repeat(
        0..,
        lift(separated_pair(
            quoted('"').and_then(filename("mat")),
            space1,
            quoted('"').and_then(filename("mat")),
        ))
        .map(|(from, to)| Override { from, to }),
    )
    .map(|overrides| TMOFile { version, overrides });

    let file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(file)
}
