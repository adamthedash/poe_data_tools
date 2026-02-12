use anyhow::{Result, anyhow};
use nom::{
    Parser,
    character::complete::space1,
    combinator::{all_consuming, opt},
    multi::many0,
    sequence::preceded,
};

use crate::file_parsers::{
    FileParser,
    lift::ToSliceParser,
    line_parser::{NomParser, Result as LResult},
    my_slice::MySlice,
    shared::{parse_bool, quoted_str, separated_array, utf16_bom_to_string, version_line2},
};

pub mod types;
use types::*;

pub struct ECFParser;

impl FileParser for ECFParser {
    type Output = EcfFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let parsed =
            parse_ecf_str(&contents).map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

        Ok(parsed)
    }
}

fn combination<'a>() -> impl NomParser<'a, EcfCombination> {
    (
        separated_array(space1, quoted_str),
        opt(preceded(space1, parse_bool)),
    )
        .map(|(et_files, bool1)| EcfCombination { et_files, bool1 })
}

fn parse_ecf_str(contents: &str) -> LResult<EcfFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let lines = MySlice(lines.as_slice());

    let parser = (
        version_line2().lift(), //
        many0(combination().lift()),
    )
        .map(|(version, combinations)| EcfFile {
            version,
            combinations,
        });

    let (_, ecf_file) = all_consuming(parser).parse_complete(lines)?;

    Ok(ecf_file)
}
