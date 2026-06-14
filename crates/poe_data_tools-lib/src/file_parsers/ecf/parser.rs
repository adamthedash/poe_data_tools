use anyhow::anyhow;
use winnow::{
    Parser,
    ascii::{dec_uint, space1},
    combinator::{opt, preceded, repeat},
};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    shared::{
        lift::lift,
        winnow::{WinnowParser, optional_filename, quoted, separated_array, version_line},
    },
};

fn combination<'a>() -> impl WinnowParser<&'a str, EcfCombination> {
    winnow::trace!(
        "combination",
        (
            separated_array(space1, quoted('"').and_then(optional_filename("et"))),
            opt(preceded(space1, dec_uint)),
        )
            .map(|(et_files, uint1)| EcfCombination { et_files, uint1 })
    )
}

pub fn parse_ecf_str(contents: &str) -> VersionedResult<EcfFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let version = lift(version_line())
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;
    //
    let mut parser = repeat(0.., lift(combination())).map(|combinations| EcfFile {
        version,
        combinations,
    });

    parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version))
}
