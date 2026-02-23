use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::space1,
    combinator::{opt, preceded, repeat},
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::lift,
    shared::winnow::{
        TraceHelper, WinnowParser, optional_filename, parse_bool, quoted, separated_array,
        version_line,
    },
};

fn combination<'a>() -> impl WinnowParser<&'a str, EcfCombination> {
    (
        separated_array(space1, quoted('"').and_then(optional_filename("et"))),
        opt(preceded(space1, parse_bool)),
    )
        .map(|(et_files, bool1)| EcfCombination { et_files, bool1 })
        .trace("combination")
}

pub fn parse_ecf_str(contents: &str) -> Result<EcfFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();

    let mut parser = (
        lift(version_line()), //
        repeat(0.., lift(combination())),
    )
        .map(|(version, combinations)| EcfFile {
            version,
            combinations,
        });

    let ecf_file = parser
        .parse(lines.as_slice())
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(ecf_file)
}
