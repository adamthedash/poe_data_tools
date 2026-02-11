use anyhow::{Result, anyhow};
use nom::{character::complete::space1, combinator::opt, sequence::preceded as P};

use crate::file_parsers::{
    FileParser,
    line_parser::{MultilineParser, Result as LResult, nom_adapter, optional, single_line},
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

fn bools<'a>() -> impl MultilineParser<'a, (bool, bool, Option<bool>, Option<bool>, Option<bool>)> {
    let line_parser = (
        parse_bool,
        P(space1, parse_bool),
        opt(P(space1, parse_bool)),
        opt(P(space1, parse_bool)),
        opt(P(space1, parse_bool)),
    );

    single_line(nom_adapter(line_parser))
}

fn parse_gt_str(contents: &str) -> LResult<GTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let (lines, name) = single_line(nom_adapter(unquoted_str))(&lines)?;

    let (lines, (bool1, bool2, bool3, bool4, bool5)) = bools()(lines)?;

    let (lines, string1) = optional(single_line(nom_adapter(quoted_str)))(lines)?;

    assert!(lines.is_empty(), "File not fully consumed: {:#?}", lines);

    let gt_file = GTFile {
        name,
        bool1,
        bool2,
        bool3,
        bool4,
        bool5,
        string1,
    };

    Ok(gt_file)
}
