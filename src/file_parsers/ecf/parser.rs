use anyhow::{Result, anyhow};
use nom::{
    Parser,
    character::complete::space1,
    combinator::{all_consuming, opt},
    multi::many0,
    sequence::preceded,
};

use super::types::*;
use crate::file_parsers::{
    lift::ToSliceParser,
    shared::{NomParser, parse_bool, quoted_str, separated_array, version_line},
    slice::Slice,
};

fn combination<'a>() -> impl NomParser<'a, EcfCombination> {
    (
        separated_array(space1, quoted_str),
        opt(preceded(space1, parse_bool)),
    )
        .map(|(et_files, bool1)| EcfCombination { et_files, bool1 })
}

pub fn parse_ecf_str(contents: &str) -> Result<EcfFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let lines = Slice(lines.as_slice());

    let parser = (
        version_line().lift(), //
        many0(combination().lift()),
    )
        .map(|(version, combinations)| EcfFile {
            version,
            combinations,
        });

    let (_, ecf_file) = all_consuming(parser)
        .parse_complete(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(ecf_file)
}
