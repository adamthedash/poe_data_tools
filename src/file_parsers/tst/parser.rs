use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, space0, space1},
    combinator::{opt, preceded, repeat, terminated},
};

use super::types::*;
use crate::file_parsers::{
    lift::lift,
    shared::winnow::{TraceHelper, WinnowParser, filename, quoted, unquoted_str},
};

fn entry<'a>() -> impl WinnowParser<&'a str, Entry> {
    (
        // NOTE: Edge case: missing space between weight & filename
        opt(terminated(dec_uint, space0)), //
        quoted('"').and_then(filename("tdt")),
        repeat(0.., preceded(space1, unquoted_str)),
    )
        .map(|(weight, tdt_file, rotations)| Entry {
            weight,
            tdt_file,
            rotations,
        })
        .trace("entry")
}

pub fn parse_tst_str(contents: &str) -> Result<TSTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();

    let mut parser = (
        repeat(
            0..,
            lift(preceded(
                ("include", space1),
                quoted('"').and_then(filename("tst")),
            )),
        ),
        repeat(0.., lift(entry())),
    )
        .map(|(includes, tdt_files)| TSTFile {
            includes,
            tdt_files,
        });

    let tst_file = parser
        .parse(lines.as_slice())
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(tst_file)
}
