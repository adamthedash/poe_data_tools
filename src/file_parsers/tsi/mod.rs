use std::collections::HashMap;

use anyhow::{Context, Result, anyhow, bail};
use nom::{
    Parser,
    branch::alt,
    character::complete::space1,
    combinator::{all_consuming, rest},
    sequence::separated_pair,
};

use crate::file_parsers::{
    line_parser::{Result as LResult, nom_adapter, single_line, take_forever},
    shared::{quoted_str, unquoted_str},
};

pub fn parse_tsi(contents: &[u8]) -> Result<HashMap<String, String>> {
    let parse_ut16 = match &contents[..2] {
        [0xff, 0xfe] => String::from_utf16le,
        [0xfe, 0xff] => String::from_utf16be,
        bytes => bail!("Invalid BOM found: {:?}", bytes),
    };

    let contents =
        parse_ut16(&contents[2..]).context("Failed to parse contents as UTF-16 string")?;

    let lut = parse_tsi_str(&contents).map_err(|e| anyhow!("Failed to parse TSI: {e:?}"))?;

    Ok(lut)
}

fn parse_tsi_str(contents: &str) -> LResult<HashMap<String, String>> {
    let lines = contents
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let line_parser = separated_pair(
        unquoted_str,
        space1,
        // Attempt to un-quote single quoted strings, otherwise just take the rest as-is
        rest.and_then(alt((all_consuming(quoted_str), rest.map(String::from)))),
    );

    let (lines, pairs) = take_forever(single_line(nom_adapter(line_parser)))(&lines)?;
    assert!(lines.is_empty(), "TSI file not fully consumed");

    Ok(HashMap::from_iter(pairs))
}
