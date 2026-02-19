use anyhow::{Result, anyhow};
use nom::{
    Parser,
    bytes::complete::tag,
    character::complete::{self, hex_digit1, space1, u32 as U},
    combinator::{all_consuming, opt, verify},
    multi::count,
    sequence::{preceded as P, separated_pair},
};

use super::types::*;
use crate::file_parsers::{
    lift_nom::{SliceParser, ToSliceParser},
    shared::{NomParser, parse_bool, separated_array, unquoted_str},
    slice::Slice,
};

fn name_hex<'a>() -> impl NomParser<'a, (String, Option<String>)> {
    let hex_parser = P(complete::char('#'), hex_digit1).map(String::from);

    (unquoted_str, opt(P(space1, hex_parser)))
}

fn gt_files<'a>() -> impl SliceParser<'a, &'a str, [String; 2]> {
    count(unquoted_str.lift(), 2)
        .map(|files| files.try_into().expect("Parser should take care of length"))
}

fn num_line<'a>() -> impl NomParser<'a, NumLine> {
    (
        U,
        P(space1, U),
        P(space1, parse_bool),
        opt(P(space1, parse_bool)),
        opt(P(space1, parse_bool)),
        opt(P(space1, parse_bool)),
    )
        .map(|(uint1, uint2, bool1, bool2, bool3, bool4)| NumLine {
            uint1,
            uint2,
            bool1,
            bool2,
            bool3,
            bool4,
        })
}

fn virtual_et_file<'a>() -> impl NomParser<'a, VirtualETFile> {
    separated_pair(unquoted_str, space1, parse_bool)
        .map(|(path, bool1)| VirtualETFile { path, bool1 })
}

fn virtual_section<'a>() -> impl SliceParser<'a, &'a str, VirtualSection> {
    (
        tag::<_, _, nom::error::Error<_>>("virtual").lift(),
        count(virtual_et_file().lift(), 2)
            .map(|files| files.try_into().expect("Parser should take care of length")),
        separated_array(space1::<_, nom::error::Error<_>>, U).lift(),
    )
        .map(
            |(_virtual_tag, virtual_et_files, virtual_rotations)| VirtualSection {
                virtual_et_files,
                virtual_rotations,
            },
        )
}

pub fn parse_et_str(contents: &str) -> Result<ETFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();
    let lines = Slice(lines.as_slice());

    let parser = (
        name_hex().lift(),
        gt_files(),
        opt(num_line().lift()),
        opt(verify(unquoted_str, |s: &str| s.ends_with(".gt")).lift()),
        opt(virtual_section()),
    )
        .map(
            |((name, hex), gt_files, num_line, gt_file2, virtual_section)| ETFile {
                name,
                hex,
                gt_files,
                num_line,
                virtual_section,
                gt_file2,
            },
        );

    let (_, et_file) = all_consuming(parser)
        .parse_complete(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(et_file)
}
