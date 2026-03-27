use anyhow::anyhow;
use winnow::{
    Parser,
    ascii::{dec_uint, space1},
    combinator::{opt, repeat, repeat_till, separated_pair},
    token::{literal, rest},
};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    shared::{
        lift::{SliceParser, lift},
        winnow::{unquoted_str, version_line},
    },
};

fn emitter<'a>() -> impl SliceParser<'a, &'a str, Emitter> {
    winnow::trace!(
        "emitter",
        (
            lift(literal("{")), //
            repeat_till::<_, _, Vec<_>, _, _, _, _>(
                .., //
                lift(separated_pair(unquoted_str, space1, rest.map(String::from))),
                lift(literal("}")),
            ),
        )
            .map(|(_, (key_values, _))| Emitter::from_iter(key_values))
    )
}

pub fn parse_trl_str(contents: &str) -> VersionedResult<TRLFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let num_emitters: usize = lift(dec_uint)
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let version = opt(lift(version_line()))
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = (
        repeat(num_emitters, emitter()),
        rest.try_map(|lines: &[&str]| {
            if lines.is_empty() {
                Ok(None)
            } else {
                serde_json::from_str(&lines.concat()).map(Some)
            }
        }),
    )
        .map(|(emitters, payload)| TRLFile {
            version,
            emitters,
            payload,
        });

    let pet_file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(version)?;

    Ok(pet_file)
}
