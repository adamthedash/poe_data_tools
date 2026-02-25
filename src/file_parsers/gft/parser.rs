use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::space1,
    combinator::{alt, cond, opt, preceded, repeat},
};

use super::types::*;
use crate::file_parsers::{
    lift::{SliceParser, lift},
    shared::winnow::{
        TraceHelper, WinnowParser, filename, quoted, quoted_str, uint as U, unquoted_str,
        version_line,
    },
};

fn file<'a>() -> impl WinnowParser<&'a str, GenFile> {
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
        .trace("file")
}

fn section<'a>(version: u32) -> impl SliceParser<'a, &'a str, Section> {
    (
        lift((quoted_str, opt(preceded(space1, U)))).trace("header"),
        cond(version == 1, lift(U)).trace("file_count"),
        repeat(0.., lift(file())).trace("files"),
    )
        .map(|((name, uint1), _, files)| Section { name, files, uint1 })
        .trace("section")
}

pub fn parse_gft_str(contents: &str) -> Result<GFTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let mut version_parser = lift(version_line());
    let version = version_parser
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = (
        cond(version == 1, lift(U)), //
        repeat(0.., section(version)),
    )
        .map(|(_num_sections, sections)| GFTFile { version, sections });

    let gft_file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(gft_file)
}
