pub mod types;

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use nom::{
    Parser,
    branch::alt,
    character::complete::space1,
    combinator::{all_consuming, rest},
    multi::many0,
    sequence::separated_pair,
};
use types::*;

use super::{
    FileParser,
    line_parser::Result as LResult,
    shared::{quoted_str, unquoted_str, utf16_bom_to_string},
};
use crate::file_parsers::{lift::ToSliceParser, my_slice::MySlice};

pub struct TSIParser;

impl FileParser for TSIParser {
    type Output = TSIFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let lut = parse_tsi_str(&contents).map_err(|e| anyhow!("Failed to parse TSI: {e:?}"))?;

        Ok(lut)
    }
}

fn parse_tsi_str(contents: &str) -> LResult<TSIFile> {
    let lines = contents
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();
    let lines = MySlice(lines.as_slice());

    let line_parser = separated_pair(
        unquoted_str,
        space1,
        // Attempt to un-quote single quoted strings, otherwise just take the rest as-is
        rest.and_then(alt((all_consuming(quoted_str), rest.map(String::from)))),
    );

    let (lines, pairs) = all_consuming(many0(line_parser.lift())).parse_complete(lines)?;

    // let (lines, pairs) = take_forever(single_line(nom_adapter(line_parser)))(&lines)?;
    assert!(lines.is_empty(), "TSI file not fully consumed");

    Ok(HashMap::from_iter(pairs))
}
