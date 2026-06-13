use anyhow::anyhow;
use winnow::{
    Parser,
    ascii::space1,
    combinator::{alt, delimited, opt, preceded as P, repeat},
    stream::{Offset, Stream},
    token::{any, literal},
};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    shared::winnow::{
        WinnowParser, quoted_str, single_quoted_str, spaces_or_comments, unquoted_str, version_line,
    },
};

fn entry<'a>() -> impl WinnowParser<&'a str, Entry> {
    winnow::trace!(
        "entry",
        (
            unquoted_str,
            (spaces_or_comments(), literal("="), spaces_or_comments()),
            alt((
                quoted_str,
                single_quoted_str,
                stack_take('{', '}').map(String::from),
                unquoted_str,
            )),
        )
            .map(|(key, _, value)| Entry { key, value })
    )
}

fn parse_struct<'a>() -> impl WinnowParser<&'a str, Struct> {
    winnow::trace!(
        "struct",
        (
            unquoted_str.verify(|s: &str| s != "client"),
            delimited(
                P(opt(spaces_or_comments()), literal("{")),
                repeat(0.., P(spaces_or_comments(), entry())),
                P(opt(spaces_or_comments()), literal("}")),
            ),
        )
            .map(|(name, entries)| Struct { name, entries })
    )
}

fn client_structs<'a>() -> impl WinnowParser<&'a str, Vec<Struct>> {
    winnow::trace!(
        "struct",
        P(
            "client",
            delimited(
                P(opt(spaces_or_comments()), literal("{")),
                repeat(0.., P(spaces_or_comments(), parse_struct())),
                P(opt(spaces_or_comments()), literal("}")),
            ),
        )
    )
}

fn stack_take<'a>(incrementer: char, decrementer: char) -> impl WinnowParser<&'a str, &'a str> {
    move |input: &mut &'a str| -> winnow::Result<&'a str> {
        let checkpoint = input.checkpoint();

        // First char must always increment
        literal(incrementer).parse_next(input)?;
        let mut counter = 1;
        while counter > 0 {
            counter += alt((
                incrementer.value(1), //
                decrementer.value(-1),
                any.value(0),
            ))
            .parse_next(input)?;
        }

        // Do some shenanigans to take first slice of input
        let mut start = "";
        start.reset(&checkpoint);

        let offset = input.offset_from(&start);
        let out = &start[..offset];

        Ok(out)
    }
}

enum StructKind {
    Base(Struct),
    Client(Vec<Struct>),
}

pub fn parse_ao_str(mut contents: &str) -> VersionedResult<AOFile> {
    let version = version_line()
        .parse_next(&mut contents)
        .map_err(|e| anyhow!("Failed to parse ao file: {e:?}"))?;

    let parser = (
        opt(P(spaces_or_comments(), literal("abstract"))),
        repeat::<_, _, Vec<_>, _, _>(
            1..,
            P(
                (spaces_or_comments(), literal("extends"), space1),
                quoted_str,
            ),
        ),
        repeat(
            0..,
            P(
                spaces_or_comments(),
                alt((
                    parse_struct().map(StructKind::Base),
                    client_structs().map(StructKind::Client),
                )),
            ),
        )
        .fold(
            || (vec![], vec![]),
            |(mut structs, mut client_structs), s| {
                match s {
                    StructKind::Base(s) => structs.push(s),
                    StructKind::Client(s) => client_structs.extend(s),
                };
                (structs, client_structs)
            },
        ),
        opt(spaces_or_comments()),
    )
        .map(
            |(is_abstract, extends, (structs, client_structs), _)| AOFile {
                version,
                is_abstract: is_abstract.is_some(),
                extends: extends.into_iter().filter(|e| e == "nothing").collect(),
                structs,
                client_structs,
            },
        );

    let mut parser = winnow::trace!("ao_file", parser);

    parser
        .parse(contents)
        .map_err(|e| anyhow!("Failed to parse ao file: {e:?}"))
        .with_version(Some(version))
}
