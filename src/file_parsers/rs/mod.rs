pub mod types;

use anyhow::{Result, anyhow};
use nom::{
    character::complete::{space1, u32 as U},
    combinator::{all_consuming, opt},
    multi::many0,
    sequence::{preceded, terminated},
};
use types::*;

use crate::file_parsers::{
    FileParser,
    line_parser::{Result as LResult, nom_adapter, single_line, take_forever},
    shared::{quoted_str, unquoted_str, utf16_bom_to_string, version_line},
};

pub struct RSParser;

impl FileParser for RSParser {
    type Output = RSFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let lut = parse_rs_str(&contents).map_err(|e| anyhow!("Failed to parse RS: {e:?}"))?;

        Ok(lut)
    }
}

fn parse_rs_str(contents: &str) -> LResult<RSFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        // Skip empty/commented lines
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();

    let (lines, version) = version_line()(&lines)?;

    let line_parser = |line| {
        let (line, weight) = opt(terminated(U, space1))(line)?;

        let (line, arm_file) = quoted_str(line)?;

        let (line, rotations) = all_consuming(many0(preceded(space1, unquoted_str)))(line)?;

        let room = Room {
            weight,
            arm_file,
            rotations,
        };

        Ok((line, room))
    };

    let (lines, rooms) = take_forever(single_line(nom_adapter(line_parser)))(lines)?;
    assert!(lines.is_empty(), "File not fully consumed");

    let room_file = RSFile { version, rooms };

    Ok(room_file)
}
