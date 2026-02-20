use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint as U, space1},
    combinator::{alt, opt, preceded as P, separated_pair},
    stream::AsChar,
    token::{literal, take_while},
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::{SliceParser, lift},
    shared::winnow::{
        TraceHelper, WinnowParser, filename, parse_bool, quoted, repeat_array, separated_array,
        unquoted, unquoted_str,
    },
};

fn name_hex<'a>() -> impl WinnowParser<&'a str, (String, Option<String>)> {
    let hex_parser = P(literal("#"), take_while(1.., AsChar::is_hex_digit)).map(String::from);

    (unquoted_str, opt(P(space1, hex_parser))).trace("name_hex")
}

fn gt_files<'a>() -> impl SliceParser<'a, &'a str, [GTFile; 2]> {
    repeat_array(lift(
        alt((
            quoted('"'), //
            unquoted(),
        ))
        .and_then(alt((
            literal("wildcard").map(|_| GTFile::Wildcard),
            filename("gt").map(GTFile::Path),
        ))),
    ))
    .trace("gt_files")
}

fn num_line<'a>() -> impl WinnowParser<&'a str, NumLine> {
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
        .trace("num_line")
}

fn virtual_et_file<'a>() -> impl WinnowParser<&'a str, VirtualETFile> {
    separated_pair(
        unquoted().and_then(filename("et")), //
        space1,
        parse_bool,
    )
    .map(|(path, bool1)| VirtualETFile { path, bool1 })
    .trace("virtual_et_file")
}

fn virtual_section<'a>() -> impl SliceParser<'a, &'a str, VirtualSection> {
    (
        lift(literal("virtual")),
        repeat_array(lift(virtual_et_file())),
        lift(separated_array(space1, U)),
    )
        .map(
            |(_virtual_tag, virtual_et_files, virtual_rotations)| VirtualSection {
                virtual_et_files,
                virtual_rotations,
            },
        )
        .trace("virtual_section")
}

pub fn parse_et_str(contents: &str) -> Result<ETFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let mut parser = (
        lift(name_hex()),
        gt_files(),
        opt(lift(num_line())),
        opt(lift(unquoted().and_then(filename("gt")))),
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

    let et_file = parser
        .parse(&lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(et_file)
}
