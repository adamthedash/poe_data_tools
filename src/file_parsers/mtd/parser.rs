use anyhow::anyhow;
use winnow::{
    Parser,
    ascii::dec_uint,
    combinator::{cond, opt, preceded as P, repeat, separated, terminated},
};

use super::{super::shared::winnow::space_or_nl1 as S, types::*};
use crate::file_parsers::shared::winnow::{
    TraceHelper, WinnowParser, parse_bool, quoted_str, version_line,
};

fn filename<'a>(extension: &str) -> impl WinnowParser<&'a str, String> {
    let ext = format!(".{extension}");

    // TODO: to_string after verify passes instead of before
    quoted_str
        .verify(move |s: &String| s.ends_with(&ext))
        .trace("filename")
}

fn entry(contents: &mut &str) -> winnow::Result<Entry> {
    (
        filename("mat"), //
        opt(P(S, filename("dlp"))),
    )
        .map(|(mat_file, dlp_file)| Entry { mat_file, dlp_file })
        .trace("entry")
        .parse_next(contents)
}

fn weights_line<'a>(num_weights: usize) -> impl WinnowParser<&'a str, (Vec<u32>, u32)> {
    (
        separated(num_weights, dec_uint::<_, u32, _>, S), //
        P(S, dec_uint),
    )
        .trace("weights_line")
}

fn group(contents: &mut &str) -> winnow::Result<Group> {
    let (name, num_a, num_b) = (
        opt(terminated(quoted_str, S)),
        dec_uint,
        P(S, dec_uint::<_, usize, _>),
    )
        .trace("group_header")
        .parse_next(contents)?;

    let entries = repeat(num_a, P(S, entry)).parse_next(contents)?;

    let weight_line = cond(
        num_a > 1, //
        P(S, weights_line(num_a)),
    )
    .parse_next(contents)?;

    let nums = cond(
        num_b > 0, //
        P(S, (dec_uint, P(S, parse_bool))),
    )
    .parse_next(contents)?;

    let extra_mat_files = repeat(num_b, P(S, filename("mat"))).parse_next(contents)?;

    let group = Group {
        name,
        entries,
        weight_line,
        extra_line: nums,
        extra_mat_files,
    };

    Ok(group)
}

pub fn parse_mtd_str(contents: &str) -> anyhow::Result<MTDFile> {
    let mut contents = contents.trim();

    let mut parser = (
        version_line(), //
        P(S, separated(1.., group, S)),
    );

    parser
        .parse_next(&mut contents)
        .map(|(version, groups)| MTDFile { version, groups })
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
}
