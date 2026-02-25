use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::space1,
    combinator::{opt, preceded as P},
};

use super::types::*;
use crate::file_parsers::{
    lift::lift,
    shared::winnow::{TraceHelper, WinnowParser, parse_bool, quoted_str, unquoted_str},
};

fn bools<'a>() -> impl WinnowParser<&'a str, (bool, bool, Option<bool>, Option<bool>, Option<bool>)>
{
    (
        parse_bool,
        P(space1, parse_bool),
        opt(P(space1, parse_bool)),
        opt(P(space1, parse_bool)),
        opt(P(space1, parse_bool)),
    )
        .trace("bools")
}

pub fn parse_gt_str(contents: &str) -> Result<GTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let mut parser = (
        lift(unquoted_str), //
        lift(bools()),
        opt(lift(quoted_str)),
    )
        .map(
            |(name, (bool1, bool2, bool3, bool4, bool5), string1)| GTFile {
                name,
                bool1,
                bool2,
                bool3,
                bool4,
                bool5,
                string1,
            },
        );

    let gt_file = parser
        .parse(lines.as_slice())
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(gt_file)
}
