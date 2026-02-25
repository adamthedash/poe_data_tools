use anyhow::{Result, anyhow};
use winnow::{Parser, ascii::space1, combinator::repeat};

use super::types::*;
use crate::file_parsers::shared::{
    lift::lift,
    winnow::{TraceHelper, WinnowParser, filename, quoted, separated_array, version_line},
};

fn combination<'a>() -> impl WinnowParser<&'a str, GcfCombination> {
    separated_array(space1, quoted('"').and_then(filename("gt")))
        .map(|gt_files| GcfCombination { gt_files })
        .trace("combination")
}

pub fn parse_gcf_str(contents: &str) -> Result<GcfFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();

    let mut parser = (
        lift(version_line()), //
        repeat(0.., lift(combination())),
    )
        .map(|(version, combinations)| GcfFile {
            version,
            combinations,
        });

    let file = parser
        .parse(lines.as_slice())
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(file)
}
