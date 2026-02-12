use anyhow::anyhow;
use nom::{
    Parser,
    character::complete::{space1, u32 as U},
    combinator::{all_consuming, cond, opt},
    multi::many0,
    sequence::preceded,
};

use crate::file_parsers::{
    FileParser,
    lift::{SliceParser, ToSliceParser},
    line_parser::{NomParser, Result as LResult},
    my_slice::MySlice,
    shared::{quoted_str, unquoted_str, utf16_bom_to_string, version_line2},
};

pub mod types;
use types::*;

pub struct GFTParser;

impl FileParser for GFTParser {
    type Output = GFTFile;

    fn parse(&self, bytes: &[u8]) -> anyhow::Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let parsed =
            parse_gft_str(&contents).map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

        Ok(parsed)
    }
}

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

fn parse_gft_str(contents: &str) -> LResult<GFTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let lines = MySlice(lines.as_slice());

    let (lines, version) = version_line2().lift().parse_complete(lines)?;

    let parser = (
        cond(version == 1, U::<_, nom::error::Error<_>>.lift()), //
        many0(section(version)),
    )
        .map(|(_num_sections, sections)| GFTFile { version, sections });

    let (_, gft_file) = all_consuming(parser).parse_complete(lines)?;

    Ok(gft_file)
}
