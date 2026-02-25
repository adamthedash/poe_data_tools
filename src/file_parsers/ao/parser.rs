use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::space1,
    combinator::{alt, delimited, opt, preceded as P, repeat},
    token::literal,
};

use super::types::*;
use crate::file_parsers::shared::winnow::{
    WinnowParser, quoted_str, single_quoted_str, spaces_or_comments, unquoted_str, version_line,
};

fn entry<'a>() -> impl WinnowParser<&'a str, Entry> {
    (
        unquoted_str,
        (spaces_or_comments(), literal("="), spaces_or_comments()),
        alt((quoted_str, single_quoted_str, unquoted_str)),
    )
        .map(|(key, _, value)| Entry { key, value })
}

fn parse_struct<'a>() -> impl WinnowParser<&'a str, Struct> {
    (
        unquoted_str,
        delimited(
            P(opt(spaces_or_comments()), literal("{")),
            repeat(0.., P(spaces_or_comments(), entry())),
            P(opt(spaces_or_comments()), literal("}")),
        ),
    )
        .map(|(name, entries)| Struct { name, entries })
}

pub fn parse_ao_str(contents: &str) -> Result<AOFile> {
    let mut parser = (
        version_line(),
        opt(P(spaces_or_comments(), literal("abstract"))),
        repeat::<_, _, Vec<_>, _, _>(
            1..,
            P(
                (spaces_or_comments(), literal("extends"), space1),
                quoted_str,
            ),
        ),
        repeat(0.., P(spaces_or_comments(), parse_struct())),
        opt(spaces_or_comments()),
    )
        .map(|(version, is_abstract, extends, structs, _)| AOFile {
            version,
            is_abstract: is_abstract.is_some(),
            extends: extends.into_iter().filter(|e| e == "nothing").collect(),
            structs,
        });

    parser
        .parse(contents)
        .map_err(|e| anyhow!("Failed to parse ao file: {e:?}"))
}
