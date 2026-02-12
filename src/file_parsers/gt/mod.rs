use anyhow::{Result, anyhow};
use nom::{Parser, character::complete::space1, combinator::opt, sequence::preceded as P};

use crate::file_parsers::{
    FileParser,
    lift::ToSliceParser,
    line_parser::{NomParser, Result as LResult},
    my_slice::MySlice,
    shared::{parse_bool, quoted_str, unquoted_str, utf16_bom_to_string},
};

pub mod types;
use types::*;

pub struct GTParser;

impl FileParser for GTParser {
    type Output = GTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let parsed = parse_gt_str(&contents).map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

        Ok(parsed)
    }
}

fn bools<'a>() -> impl NomParser<'a, (bool, bool, Option<bool>, Option<bool>, Option<bool>)> {
    (
        parse_bool,
        P(space1, parse_bool),
        opt(P(space1, parse_bool)),
        opt(P(space1, parse_bool)),
        opt(P(space1, parse_bool)),
    )
}

fn parse_gt_str(contents: &str) -> LResult<GTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();
    let lines = MySlice(lines.as_slice());

    let mut parser = (
        unquoted_str.lift(), //
        bools().lift(),
        opt(quoted_str.lift()),
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

    let (_, gt_file) = parser.parse_complete(lines)?;

    Ok(gt_file)
}
