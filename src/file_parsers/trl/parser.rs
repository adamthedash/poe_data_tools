use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_uint, space1},
    combinator::{opt, repeat, repeat_till, separated_pair},
    token::{literal, rest},
};

use super::types::*;
use crate::file_parsers::{
    lift::{SliceParser, lift},
    shared::winnow::{TraceHelper, unquoted_str, version_line},
};

fn emitter<'a>() -> impl SliceParser<'a, &'a str, Emitter> {
    (
        lift(literal("{")), //
        repeat_till::<_, _, Vec<_>, _, _, _, _>(
            .., //
            lift(separated_pair(unquoted_str, space1, rest.map(String::from))),
            lift(literal("}")),
        ),
    )
        .map(|(_, (key_values, _))| Emitter::from_iter(key_values))
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
