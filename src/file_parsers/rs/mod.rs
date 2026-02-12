use crate::file_parsers::lift::ToSliceParser;
pub mod types;

use anyhow::{Result, anyhow};
use nom::{
    Parser,
    character::complete::{space0, space1, u32 as U},
    combinator::{all_consuming, opt},
    multi::many0,
    sequence::{preceded, terminated},
};
use types::*;

use crate::file_parsers::{
    FileParser,
    line_parser::{NomParser, Result as LResult},
    my_slice::MySlice,
    shared::{quoted_str, unquoted_str, utf16_bom_to_string, version_line2},
};

pub struct RSParser;

impl FileParser for RSParser {
    type Output = RSFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let lut = parse_rs_str(&contents).map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

        Ok(lut)
    }
}

fn room<'a>() -> impl NomParser<'a, Room> {
    (
        opt(terminated(U, space0)),
        quoted_str,
        all_consuming(many0(preceded(space1, unquoted_str))),
    )
        .map(|(weight, arm_file, rotations)| Room {
            weight,
            arm_file,
            rotations,
        })
}

fn parse_rs_str(contents: &str) -> LResult<RSFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        // Skip empty/commented lines
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let lines = MySlice(lines.as_slice());

    let parser = (
        version_line2().lift(), //
        many0(room().lift()),
    )
        .map(|(version, rooms)| RSFile { version, rooms });

    let (_, room_file) = all_consuming(parser).parse_complete(lines)?;

    Ok(room_file)
}
