use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::space1,
    combinator::{alt, eof, repeat, separated_pair, terminated},
    error::ContextError,
    token::rest,
};

use super::types::TSIFile;
use crate::file_parsers::{
    lift::lift,
    shared::winnow::{TraceHelper, quoted_str, unquoted_str},
};

fn key_value<'a>() -> impl Parser<&'a str, (String, String), ContextError> {
    separated_pair(
        unquoted_str,
        space1,
        // Attempt to un-quote single quoted strings, otherwise just take the rest as-is
        alt((
            terminated(quoted_str, eof), //
            rest.map(String::from),
        )),
    )
    .trace("key_value")
}

pub fn parse_tsi_str(contents: &str) -> Result<TSIFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();

    let mut parser = repeat(0.., lift(key_value()));

    let tsi_file = parser
        .parse(lines.as_slice())
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(tsi_file)
}
