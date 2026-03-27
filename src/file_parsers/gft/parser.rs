use anyhow::anyhow;
use winnow::{
    Parser,
    ascii::space1,
    combinator::{alt, cond, opt, preceded, repeat},
};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    shared::{
        lift::{SliceParser, lift},
        winnow::{
            WinnowParser, filename, quoted, quoted_str, uint as U, unquoted_str, version_line,
        },
    },
};

fn file<'a>() -> impl WinnowParser<&'a str, GenFile> {
    winnow::trace!(
        "file",
        (
            U,
            preceded(
                space1,
                quoted('"').and_then(alt((
                    filename("arm"), //
                    filename("tdt"),
                ))),
            ),
            repeat(0.., preceded(space1, unquoted_str)),
        )
            .map(|(weight, path, rotations)| GenFile {
                weight,
                path,
                rotations,
            })
    )
}

fn section<'a>(version: u32) -> impl SliceParser<'a, &'a str, Section> {
    winnow::trace!(
        "section",
        (
            winnow::trace!("header", lift((quoted_str, opt(preceded(space1, U))))),
            winnow::trace!("file_count", cond(version == 1, lift(U))),
            winnow::trace!("files", repeat(0.., lift(file()))),
        )
            .map(|((name, uint1), _, files)| Section { name, files, uint1 })
    )
}

pub fn parse_gft_str(contents: &str) -> VersionedResult<GFTFile> {
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
        cond(version == 1, lift(U)), //
        repeat(0.., section(version)),
    )
        .map(|(_num_sections, sections)| GFTFile { version, sections });

    parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version))
}
