use anyhow::{Result, anyhow};
use nom::{Parser, character::complete::space1, combinator::opt, sequence::preceded};

use crate::file_parsers::{
    FileParser,
    line_parser::{Result as LResult, nom_adapter, single_line, take_forever},
    shared::{parse_bool, quoted_str, separated_array, utf16_bom_to_string, version_line},
};

pub mod types;
use types::*;

pub struct ECFParser;

impl FileParser for ECFParser {
    type Output = EcfFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let parsed = parse_ecf_str(&contents).map_err(|e| anyhow!("Failed to parse ECF: {e:?}"))?;

        Ok(parsed)
    }
}

fn parse_ecf_str(contents: &str) -> LResult<EcfFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();

    let (lines, version) = version_line()(&lines)?;

    let line_parser = (
        separated_array(space1, quoted_str),
        opt(preceded(space1, parse_bool)),
    )
        .map(|(et_files, bool1)| EcfCombination { et_files, bool1 });

    let (lines, combinations) = take_forever(single_line(nom_adapter(line_parser)))(lines)?;
    assert!(lines.is_empty(), "File not fully consumed");

    let ecf_file = EcfFile {
        version,
        combinations,
    };

    Ok(ecf_file)
}
