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
    shared::{quoted_str, unquoted_str, utf16_bom_to_string},
};
use crate::file_parsers::{lift::ToSliceParser, shared::NomParser, slice::Slice};

pub struct TSIParser;

impl FileParser for TSIParser {
    type Output = TSIFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        parse_tsi_str(&contents)
    }
}

fn key_value<'a>() -> impl NomParser<'a, (String, String)> {
    separated_pair(
        unquoted_str,
        space1,
        // Attempt to un-quote single quoted strings, otherwise just take the rest as-is
        rest.and_then(alt((all_consuming(quoted_str), rest.map(String::from)))),
    )
}

fn parse_tsi_str(contents: &str) -> Result<TSIFile> {
    let lines = contents
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();
    let lines = Slice(lines.as_slice());

    let parser = many0(key_value().lift()).map(HashMap::from_iter);

    let (_, tsi_file) = all_consuming(parser)
        .parse_complete(lines)
        .map_err(|e| anyhow!("Failed to parse TSI: {e:?}"))?;

    Ok(tsi_file)
}
