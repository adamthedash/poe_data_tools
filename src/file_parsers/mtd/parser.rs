use anyhow::anyhow;
use winnow::{
    Parser,
    ascii::{dec_uint, space0},
    combinator::{cond, opt, preceded as P, repeat, separated, terminated},
};

use super::{super::shared::winnow::spaces_or_comments as S, types::*};
use crate::file_parsers::shared::winnow::{
    TraceHelper, WinnowParser, filename, parse_bool, quoted, quoted_str, version_line,
};

fn entry(contents: &mut &str) -> winnow::Result<Entry> {
    (
        quoted('"').and_then(filename("mat")),
        repeat(
            0..,
            P(
                // NOTE Edge case: Missing space between strings here
                space0, //
                quoted('"').and_then(filename("dlp")),
            ),
        ),
    )
        .map(|(mat_file, dlp_files)| Entry {
            mat_file,
            dlp_files,
        })
        .trace("entry")
        .parse_next(contents)
}

fn weights_line<'a>(num_weights: usize) -> impl WinnowParser<&'a str, (Vec<u32>, u32)> {
    (
        separated(num_weights, dec_uint::<_, u32, _>, S()), //
        P(S(), dec_uint),
    )
        .trace("weights_line")
}

fn group(contents: &mut &str) -> winnow::Result<Group> {
    let (name, num_a, num_b) = (
        opt(terminated(quoted_str, S())),
        dec_uint,
        P(S(), dec_uint::<_, usize, _>),
    )
        .trace("group_header")
        .parse_next(contents)?;

    let entries = repeat(num_a, P(S(), entry)).parse_next(contents)?;

    let weight_line = cond(
        num_a > 1, //
        P(S(), weights_line(num_a)),
    )
    .parse_next(contents)?;

    let nums = cond(
        num_b > 0, //
        P(S(), (dec_uint, P(S(), parse_bool))),
    )
    .parse_next(contents)?;

    let extra_entries = repeat(num_b, P(S(), entry)).parse_next(contents)?;

    let group = Group {
        name,
        entries,
        weight_line,
        extra_line: nums,
        extra_entries,
    };

    Ok(group)
}

pub fn parse_mtd_str(contents: &str) -> anyhow::Result<MTDFile> {
    let mut contents = contents.trim();

    let mut parser = (
        version_line(), //
        P(S(), separated(1.., group, S())),
    );

    parser
        .parse_next(&mut contents)
        .map(|(version, groups)| MTDFile { version, groups })
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
}
