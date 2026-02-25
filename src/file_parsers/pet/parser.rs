use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::dec_uint,
    binary::length_repeat,
    combinator::{opt, repeat_till},
    token::{literal, rest},
};

use super::types::*;
use crate::file_parsers::{
    lift::{SliceParser, lift},
    shared::winnow::{TraceHelper, optional_filename, quoted, unquoted_str, version_line},
};

fn emitter<'a>() -> impl SliceParser<'a, &'a str, Emitter> {
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
        .trace("emitter")
}

pub fn parse_pet_str(contents: &str) -> Result<PETFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let mut parser = (
        opt(lift(version_line())), //
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
        .map(|(version, emitters, payload)| PETFile {
            version,
            emitters,
            payload,
        });

    let pet_file = parser
        .parse(lines.as_slice())
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(pet_file)
}
