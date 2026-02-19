use anyhow::{Result, anyhow};
use nom::{
    Parser,
    character::complete::{space1, u32 as U},
    combinator::{all_consuming, cond, opt},
    multi::many0,
    sequence::preceded,
};

use super::types::*;
use crate::file_parsers::{
    lift::{SliceParser, ToSliceParser},
    shared::{NomParser, quoted_str, unquoted_str, version_line},
    slice::Slice,
};

fn file<'a>() -> impl NomParser<'a, GenFile> {
    (
        U,
        preceded(space1, quoted_str),
        all_consuming(many0(preceded(space1, unquoted_str))),
    )
        .map(|(weight, path, rotations)| GenFile {
            weight,
            path,
            rotations,
        })
}

fn section<'a>(version: u32) -> impl SliceParser<'a, &'a str, Section> {
    (
        quoted_str.and(opt(preceded(space1, U))).lift(),
        cond(version == 1, U::<_, nom::error::Error<_>>.lift()),
        many0(file().lift()),
    )
        .map(|((name, uint1), _, files)| Section { name, files, uint1 })
}

pub fn parse_gft_str(contents: &str) -> Result<GFTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let lines = Slice(lines.as_slice());

    let (lines, version) = version_line()
        .lift()
        .parse_complete(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let parser = (
        cond(version == 1, U::<_, nom::error::Error<_>>.lift()), //
        many0(section(version)),
    )
        .map(|(_num_sections, sections)| GFTFile { version, sections });

    let (_, gft_file) = all_consuming(parser)
        .parse_complete(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(gft_file)
}
