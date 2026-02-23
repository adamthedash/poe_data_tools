use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::dec_uint,
    combinator::{opt, repeat, repeat_till},
    token::{literal, rest},
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::{SliceParser, lift},
    shared::winnow::{TraceHelper, version_line},
};

fn emitter<'a>() -> impl SliceParser<'a, &'a str, Emitter> {
    (
        lift(literal("{")), //
        repeat_till::<_, _, Vec<_>, _, _, _, _>(
            .., //
            lift(rest),
            lift(literal("}")),
        ),
    )
        .map(|(_, (contents, _))| Emitter {
            key_values: contents.join("\n"),
        })
        .trace("emitter")
}

pub fn parse_trl_str(contents: &str) -> Result<TRLFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let num_emitters: usize = lift(dec_uint)
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = (
        opt(lift(version_line())), //
        repeat(num_emitters, emitter()),
        rest.try_map(|lines: &[&str]| {
            if lines.is_empty() {
                Ok(None)
            } else {
                serde_json::from_str(&lines.concat()).map(Some)
            }
        }),
    )
        .map(|(version, emitters, payload)| TRLFile {
            version,
            emitters,
            payload,
        });

    let pet_file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(pet_file)
}
