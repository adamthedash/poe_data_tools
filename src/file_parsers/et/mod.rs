use anyhow::{Result, anyhow};
use nom::{
    IResult, Parser,
    bytes::complete::tag,
    character::complete::{self, hex_digit1, space1, u32 as U},
    combinator::{opt, verify},
    sequence::{Tuple, preceded as P, separated_pair},
};

use crate::file_parsers::{
    FileParser,
    et::types::{ETFile, NumLine, VirtualETFile, VirtualSection},
    line_parser::{
        MultilineParser, Result as LResult, nom_adapter, optional, repeated, single_line,
    },
    shared::{parse_bool, separated_array, unquoted_str, utf16_bom_to_string},
};

pub mod types;

pub struct ETParser;

impl FileParser for ETParser {
    type Output = ETFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let parsed = parse_et_str(&contents).map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

        Ok(parsed)
    }
}

fn name_hex<'a>() -> impl MultilineParser<'a, (String, Option<String>)> {
    single_line(nom_adapter(|line| {
        let (line, name) = unquoted_str(line)?;

        let hex_parser = P(complete::char('#'), hex_digit1).map(String::from);
        let (line, hex) = opt(P(space1, hex_parser))(line)?;

        Ok((line, (name, hex)))
    }))
}

fn gt_files<'a>() -> impl MultilineParser<'a, [String; 2]> {
    |lines| {
        let (lines, gt_files) = repeated(single_line(nom_adapter(unquoted_str)), 2)(lines)?;
        let gt_files = gt_files
            .try_into()
            .expect("Parser should take care of length");

        Ok((lines, gt_files))
    }
}

fn num_line<'a>() -> impl MultilineParser<'a, NumLine> {
    single_line(nom_adapter(|line| -> IResult<_, _> {
        let (line, (uint1, uint2, bool1, bool2, bool3, bool4)) = (
            U,
            P(space1, U),
            P(space1, parse_bool),
            opt(P(space1, parse_bool)),
            opt(P(space1, parse_bool)),
            opt(P(space1, parse_bool)),
        )
            .parse(line)?;

        let num_line = NumLine {
            uint1,
            uint2,
            bool1,
            bool2,
            bool3,
            bool4,
        };

        Ok((line, num_line))
    }))
}

fn virtual_section<'a>() -> impl MultilineParser<'a, VirtualSection> {
    |lines| {
        let (lines, _virtual_tag) = single_line(nom_adapter(tag("virtual")))(lines)?;

        let (lines, virtual_et_files) = repeated(
            single_line(nom_adapter(
                separated_pair(unquoted_str, space1, parse_bool)
                    .map(|(path, bool1)| VirtualETFile { path, bool1 }),
            )),
            2,
        )(lines)?;
        let virtual_et_files = virtual_et_files
            .try_into()
            .expect("Parser should take care of length");

        let (lines, virtual_rotations) =
            single_line(nom_adapter(separated_array(space1, U)))(lines)?;

        let virtual_section = VirtualSection {
            virtual_et_files,
            virtual_rotations,
        };

        Ok((lines, virtual_section))
    }
}

fn parse_et_str(contents: &str) -> LResult<ETFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let (lines, (name, hex)) = name_hex()(&lines)?;

    let (lines, gt_files) = gt_files()(lines)?;

    let (lines, num_line) = optional(num_line())(lines)?;

    let (lines, gt_file2) = optional(single_line(nom_adapter(verify(unquoted_str, |s: &str| {
        s.ends_with(".gt")
    }))))(lines)?;

    let (lines, virtual_section) = optional(virtual_section())(lines)?;
    assert!(lines.is_empty(), "File not fully consumed: {:#?}", lines);

    let gt_file = ETFile {
        name,
        hex,
        gt_files,
        num_line,
        virtual_section,
        gt_file2,
    };

    Ok(gt_file)
}
