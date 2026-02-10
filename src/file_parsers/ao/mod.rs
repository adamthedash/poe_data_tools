pub mod types;
use anyhow::{Result, anyhow, ensure};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{space1, u32 as U},
    combinator::{all_consuming, opt},
    multi::{many0, many1},
    sequence::{Tuple, delimited, preceded as P, tuple},
};
use types::*;

use crate::file_parsers::{
    FileParser,
    shared::{quoted_str, single_quoted_str, space_or_nl1, unquoted_str, utf16_bom_to_string},
};

pub struct AOParser;

impl FileParser for AOParser {
    type Output = AOFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let (contents, parsed) =
            parse_ao_str(&contents).map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;
        ensure!(contents.is_empty(), "Input remaining: {:?}", contents);

        Ok(parsed)
    }
}

fn entry(contents: &str) -> IResult<&str, Entry> {
    let (contents, key) = unquoted_str(contents)?;

    let (contents, _) = (spaces_or_comments, tag("="), spaces_or_comments).parse(contents)?;

    let (contents, value) = alt((quoted_str, single_quoted_str, unquoted_str))(contents)?;

    let entry = Entry { key, value };

    Ok((contents, entry))
}

fn parse_struct(contents: &str) -> IResult<&str, Struct> {
    let (contents, name) = unquoted_str(contents)?;

    let (contents, entries) = delimited(
        P(opt(spaces_or_comments), tag("{")),
        many0(P(spaces_or_comments, entry)),
        P(opt(spaces_or_comments), tag("}")),
    )(contents)?;

    let s = Struct { name, entries };

    Ok((contents, s))
}

/// /* multiline comments */
fn comment_multiline(contents: &str) -> IResult<&str, &str> {
    delimited(tag("/*"), take_until("*/"), tag("*/"))(contents)
}

/// // Single line comment
fn comment_single_line(contents: &str) -> IResult<&str, &str> {
    let (contents, _) = tag("//")(contents)?;

    let (contents, comment) = take_while(|c| !(c == '\r' || c == '\n'))(contents)?;

    Ok((contents, comment))
}

/// Some combination of spaces, newlines, or comments, at least 1
fn spaces_or_comments(contents: &str) -> IResult<&str, String> {
    let part_parser = alt((space_or_nl1, comment_multiline, comment_single_line));

    let (contents, parts) = many1(part_parser)(contents)?;

    Ok((contents, parts.concat()))
}

fn parse_ao_str(contents: &str) -> IResult<&str, AOFile> {
    let contents = contents.trim();

    // TODO: comments before?
    let (contents, version) = P(tag("version "), U)(contents)?;

    let (contents, is_abstract) = opt(P(spaces_or_comments, tag("abstract")))(contents)?;
    let is_abstract = is_abstract.is_some();

    let (contents, extends) = many1(P(
        tuple((spaces_or_comments, tag("extends"), space1)),
        quoted_str,
    ))(contents)?;
    let extends = extends.into_iter().filter(|e| e != "nothing").collect();

    let (contents, structs) = many0(P(spaces_or_comments, parse_struct))(contents)?;

    // Remaining data must be junk
    let (contents, _) = all_consuming(opt(spaces_or_comments))(contents)?;
    assert!(contents.is_empty());

    let ao_file = AOFile {
        version,
        is_abstract,
        extends,
        structs,
    };

    Ok((contents, ao_file))
}
