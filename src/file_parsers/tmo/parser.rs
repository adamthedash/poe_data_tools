use anyhow::anyhow;
use winnow::{
    Parser,
    ascii::space0,
    combinator::{repeat, separated_pair, terminated},
    token::rest,
};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    shared::{
        lift::lift,
        winnow::{filename, quoted, version_line},
    },
};

pub fn parse_tmo_str(contents: &str) -> VersionedResult<TMOFile> {
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
        lift(terminated(
            separated_pair(
                quoted('"').and_then(filename("mat")),
                // NOTE: Edge case: missing space between files
                space0,
                quoted('"').and_then(filename("mat")),
            ),
            // NOTE: Edge case: Sometimes some extra crap at the end
            rest.verify(|crap: &str| {
                if !crap.is_empty() {
                    log::debug!("Extra crap found: {crap:?}");
                }
                true
            }),
        ))
        .map(|(from, to)| Override { from, to }),
    )
    .map(|overrides| TMOFile { version, overrides });

    parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version))
}
