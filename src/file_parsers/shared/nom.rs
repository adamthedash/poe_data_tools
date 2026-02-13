use itertools::Itertools;
use nom::{
    IResult, Input, Parser,
    bytes::complete::{tag, take_till1, take_until},
    character::complete::{char as C, u32 as U},
    multi::count,
    sequence::{delimited, preceded as P},
};

pub trait NomParser<'a, T> = nom::Parser<&'a str, Output = T, Error = nom::error::Error<&'a str>>;

/// Parses a 0/1 as a bool
pub fn parse_bool(line: &str) -> IResult<&str, bool> {
    let (rest, item) = U(line)?;

    let item = match item {
        0 => false,
        1 => true,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                line,
                // No better option here
                nom::error::ErrorKind::Digit,
            )));
        }
    };

    Ok((rest, item))
}

/// Parses a u32, ensuring that it hasn't just parsed the first digit of what's actually a float
pub fn safe_u32(line: &str) -> IResult<&str, u32> {
    let (line, uint) = U(line)?;
    if line.starts_with(".") {
        // fail - actually a float
        return Err(nom::Err::Error(nom::error::Error::new(
            line,
            nom::error::ErrorKind::Digit,
        )));
    }

    Ok((line, uint))
}

/// " \t\r\n" - at least 1
pub fn space_or_nl1(input: &str) -> IResult<&str, &str> {
    input.split_at_position1_complete(
        |c| !(c == ' ' || c == '\t' || c == '\r' || c == '\n'),
        nom::error::ErrorKind::Space,
    )
}

/// " \t\r\n" - 0 or more
pub fn space_or_nl0(input: &str) -> IResult<&str, &str> {
    input.split_at_position_complete(|c| !(c == ' ' || c == '\t' || c == '\r' || c == '\n'))
}

pub fn quoted_str(input: &str) -> IResult<&str, String> {
    delimited(C('"'), take_until("\""), C('"'))
        .map(String::from)
        .parse(input)
}

pub fn single_quoted_str(input: &str) -> IResult<&str, String> {
    delimited(C('\''), take_until("'"), C('\''))
        .map(String::from)
        .parse(input)
}

pub fn unquoted_str(input: &str) -> IResult<&str, String> {
    take_till1(|c: char| c.is_whitespace())
        .map(String::from)
        .parse(input)
}

pub fn version_line<'a>() -> impl NomParser<'a, u32> {
    P(tag("version "), U)
}

/// nom::sequence::separated_list but exact sized
pub fn separated_array<const N: usize, I, S, F>(
    sep: S,
    item: F,
) -> impl Parser<I, Output = [F::Output; N], Error = F::Error>
where
    I: Input,
    S: Parser<I>,
    F: Parser<I, Error = S::Error> + Clone,
{
    (
        item.clone(), //
        count(P(sep, item), N - 1),
    )
        .map(|(first, rest)| {
            std::iter::once(first)
                .chain(rest)
                .collect_array()
                .unwrap_or_else(|| unreachable!())
        })
}
