pub mod types;
use anyhow::{Result, anyhow, ensure};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{space1, u32 as U},
    combinator::{all_consuming, opt},
    multi::{many0, many1},
    sequence::{delimited, preceded as P},
};
use types::*;

use crate::file_parsers::{
    FileParser,
    line_parser::NomParser,
    shared::{quoted_str, single_quoted_str, space_or_nl1, unquoted_str, utf16_bom_to_string},
};

pub struct AOParser;

impl FileParser for AOParser {
    type Output = AOFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let (contents, parsed) =
            parse_ao_str(contents.trim()).map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;
        ensure!(contents.is_empty(), "Input remaining: {:?}", contents);

        Ok(parsed)
    }
}

fn entry<'a>() -> impl NomParser<'a, Entry> {
    (
        unquoted_str,
        (spaces_or_comments(), tag("="), spaces_or_comments()),
        alt((quoted_str, single_quoted_str, unquoted_str)),
    )
        .map(|(key, _, value)| Entry { key, value })
}

fn parse_struct<'a>() -> impl NomParser<'a, Struct> {
    (
        unquoted_str,
        delimited(
            P(opt(spaces_or_comments()), tag("{")),
            many0(P(spaces_or_comments(), entry())),
            P(opt(spaces_or_comments()), tag("}")),
        ),
    )
        .map(|(name, entries)| Struct { name, entries })
}

/// /* multiline comments */
fn comment_multiline<'a>() -> impl NomParser<'a, &'a str> {
    delimited(tag("/*"), take_until("*/"), tag("*/"))
}

/// // Single line comment
fn comment_single_line<'a>() -> impl NomParser<'a, &'a str> {
    P(tag("//"), take_while(|c| !(c == '\r' || c == '\n')))
}

/// Some combination of spaces, newlines, or comments, at least 1
fn spaces_or_comments<'a>() -> impl NomParser<'a, String> {
    let part_parser = alt((space_or_nl1, comment_multiline(), comment_single_line()));

    many1(part_parser).map(|x| x.concat())
}

fn parse_ao_str(contents: &str) -> IResult<&str, AOFile> {
    (
        P(tag("version "), U),
        opt(P(spaces_or_comments(), tag("abstract"))),
        many1(P(
            (spaces_or_comments(), tag("extends"), space1),
            quoted_str,
        )),
        many0(P(spaces_or_comments(), parse_struct())),
        // Trailing junk
        all_consuming(opt(spaces_or_comments())),
    )
        .map(|(version, is_abstract, extends, structs, _)| AOFile {
            version,
            is_abstract: is_abstract.is_some(),
            extends: extends.into_iter().filter(|e| e == "nothing").collect(),
            structs,
        })
        .parse_complete(contents)
}
