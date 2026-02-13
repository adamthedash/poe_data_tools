use std::collections::HashMap;

use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::space1,
    combinator::{repeat, separated_pair},
    error::ContextError,
    token::rest,
};

use super::types::TSIFile;
use crate::file_parsers::{
    lift_winnow::lift,
    shared::winnow::{TraceHelper, unquoted_str},
};

fn key_value<'a>() -> impl Parser<&'a str, (String, String), ContextError> {
    separated_pair(
        unquoted_str,
        space1,
        // Attempt to un-quote single quoted strings, otherwise just take the rest as-is
        rest.map(String::from),
    )
    .trace("key_value")
}

pub fn parse_tsi_str(contents: &str) -> Result<TSIFile> {
    let lines = contents
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let parser = repeat(0.., key_value().map(|(k, v)| (k, v)));
    let mut line_parser = lift(parser);

    let tsi_file = line_parser
        .parse_next(&mut lines)
        .map(|vec: Vec<(String, String)>| HashMap::from_iter(vec))
        .map_err(|e| anyhow!("Failed to parse TSI: {e:?}"))?;

    Ok(tsi_file)
}
