pub mod types;

use anyhow::{Result, anyhow};
use nom::{
    Parser,
    character::complete::{space1, u32 as U},
    combinator::opt,
    sequence::{pair, terminated},
};
use types::*;

use crate::file_parsers::{
    line_parser::{Result as LResult, nom_adapter, single_line, take_forever},
    shared::{quoted_str, utf16_bom_to_string},
};

pub fn parse_rs(contents: &[u8]) -> Result<RSFile> {
    let contents = utf16_bom_to_string(contents)?;

    let lut = parse_rs_str(&contents).map_err(|e| anyhow!("Failed to parse RS: {e:?}"))?;

    Ok(lut)
}

fn parse_rs_str(contents: &str) -> LResult<RSFile> {
    let lines = contents
        .lines()
        // Skip empty/commented lines
        .filter(|l| !l.is_empty() || l.starts_with(r"\\!"))
        .collect::<Vec<_>>();

    let line_parser = pair(opt(terminated(U, space1)), quoted_str)
        .map(|(weight, arm_file)| Room { weight, arm_file });

    let (lines, rooms) = take_forever(single_line(nom_adapter(line_parser)))(&lines)?;
    assert!(lines.is_empty(), "File not fully consumed");

    Ok(rooms)
}
