use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::space1,
    combinator::{alt, repeat, separated_pair},
    error::ContextError,
};

use super::types::TSIFile;
use crate::file_parsers::{
    lift_winnow::lift,
    shared::winnow::{TraceHelper, quoted_str, unquoted_str},
};

fn key_value<'a>() -> impl Parser<&'a str, (String, String), ContextError> {
    separated_pair(
        unquoted_str,
        space1,
        // Attempt to un-quote single quoted strings, otherwise just take the rest as-is
        alt((quoted_str, unquoted_str)),
    )
    .trace("key_value")
}

pub fn parse_tsi_str(contents: &str) -> Result<TSIFile> {
    let lines = contents
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let mut parser = repeat(0.., lift(key_value()));

    let tsi_file = parser
        .parse(lines.as_slice())
        .map_err(|e| anyhow!("Failed to parse TSI: {e:?}"))?;

    Ok(tsi_file)
}
