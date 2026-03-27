use anyhow::anyhow;
use winnow::{
    Parser,
    ascii::dec_uint,
    binary::length_repeat,
    combinator::{opt, repeat_till},
    token::{literal, rest},
};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    shared::{
        lift::{SliceParser, lift},
        winnow::{optional_filename, quoted, unquoted_str, version_line},
    },
};

fn emitter<'a>() -> impl SliceParser<'a, &'a str, Emitter> {
    winnow::trace!(
        "emitter",
        (
            lift(literal("{")), //
            lift(unquoted_str),
            lift(quoted('"').and_then(optional_filename("mat"))),
            repeat_till::<_, _, Vec<_>, _, _, _, _>(
                .., //
                lift(rest),
                lift(literal("}")),
            ),
        )
            .map(|(_, emitter_type, material, (contents, _))| Emitter {
                emitter_type,
                material,
                key_values: contents.join("\n"),
            })
    )
}

pub fn parse_pet_str(contents: &str) -> VersionedResult<PETFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let mut lines = lines.as_slice();

    let version = opt(lift(version_line()))
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = (
        length_repeat(
            lift(dec_uint::<_, u32, _>), //
            emitter(),
        ),
        rest.try_map(|lines: &[&str]| {
            if lines.is_empty() {
                Ok(None)
            } else {
                serde_json::from_str(&lines.concat()).map(Some)
            }
        }),
    )
        .map(|(emitters, payload)| PETFile {
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
