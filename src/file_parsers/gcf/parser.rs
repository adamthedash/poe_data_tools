use anyhow::anyhow;
use winnow::{Parser, ascii::space1, combinator::repeat};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    shared::{
        lift::lift,
        winnow::{WinnowParser, filename, quoted, separated_array, version_line},
    },
};

fn combination<'a>() -> impl WinnowParser<&'a str, GcfCombination> {
    winnow::trace!(
        "combination",
        separated_array(space1, quoted('"').and_then(filename("gt")))
            .map(|gt_files| GcfCombination { gt_files })
    )
}

pub fn parse_gcf_str(contents: &str) -> VersionedResult<GcfFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let version = lift(version_line())
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = repeat(0.., lift(combination())).map(|combinations| GcfFile {
        version,
        combinations,
    });

    let file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version))?;

    Ok(file)
}
