use std::fmt::Display;

use winnow::{
    Parser,
    ascii::dec_uint,
    combinator::{alt, delimited, eof, preceded, repeat, separated, trace},
    error::{ContextError, ParserError},
    stream::{AsChar, Stream},
    token::{literal, rest, take_till, take_until, take_while},
};

pub trait WinnowParser<I, T> = Parser<I, T, ContextError>;

/// Parses a 0/1 as a bool
pub fn parse_bool(input: &mut &str) -> winnow::Result<bool> {
    let parser = |input: &mut &str| {
        let uint: u32 = dec_uint.parse_next(input)?;

        match uint {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ContextError::new()),
        }
    };

    parser.trace("parse_bool").parse_next(input)
}

/// Parses a u32, ensuring that it hasn't just parsed the first digit of what's actually a float
pub fn safe_u32(input: &mut &str) -> winnow::Result<u32> {
    let parser = |input: &mut &str| {
        let uint = dec_uint.parse_next(input)?;
        if input.starts_with('.') {
            // fail - actually a float
            Err(ContextError::new())
        } else {
            Ok(uint)
        }
    };

    parser.trace("safe_u32").parse_next(input)
}

/// " \t\r\n" - at least 1
pub fn space_or_nl1<'a>(input: &mut &'a str) -> winnow::Result<&'a str> {
    take_while(1.., |c: char| {
        c == ' ' || c == '\t' || c == '\r' || c == '\n'
    })
    .trace("space_or_nl1")
    .parse_next(input)
}

/// " \t\r\n" - 0 or more
pub fn space_or_nl0<'a>(input: &mut &'a str) -> winnow::Result<&'a str> {
    take_while(0.., |c: char| {
        c == ' ' || c == '\t' || c == '\r' || c == '\n'
    })
    .trace("space_or_nl0")
    .parse_next(input)
}

pub fn quoted<'a>(quote: char) -> impl WinnowParser<&'a str, &'a str> {
    delimited(
        quote, //
        take_until(0.., quote),
        quote,
    ) //
    .trace(format!("quoted {quote:?}"))
}

pub fn unquoted<'a>() -> impl WinnowParser<&'a str, &'a str> {
    take_till(1.., AsChar::is_space).trace("unquoted")
}

pub fn quoted_str(input: &mut &str) -> winnow::Result<String> {
    quoted('"')
        .map(String::from)
        .trace("quoted_str")
        .parse_next(input)
}

pub fn single_quoted_str(input: &mut &str) -> winnow::Result<String> {
    quoted('\"')
        .map(String::from)
        .trace("single_quoted_str")
        .parse_next(input)
}

pub fn unquoted_str(input: &mut &str) -> winnow::Result<String> {
    unquoted()
        .map(String::from)
        .trace("unquoted_str")
        .parse_next(input)
}

/// Designed for use with Parser::and_then
pub fn filename<'a>(extension: &str) -> impl WinnowParser<&'a str, String> {
    let ext = format!(".{extension}");

    rest.verify(move |s: &str| s.ends_with(&ext))
        .map(String::from)
        .trace("filename")
}

/// Filename with the provided extension, or empty string
/// Designed for use with Parser::and_then
pub fn optional_filename<'a>(extension: &str) -> impl WinnowParser<&'a str, Option<String>> {
    alt((
        eof.map(|_| None), //
        filename(extension).map(Some),
    ))
    .trace("optional_filename")
}

pub fn version_line<'a>() -> impl WinnowParser<&'a str, u32> {
    preceded(literal("version "), dec_uint).trace("version_line")
}

/// winnow::combinator::multi::separated but exact sized
pub fn separated_array<const N: usize, I, P, S, PO, SO>(
    sep: S,
    item: P,
) -> impl WinnowParser<I, [PO; N]>
where
    I: Stream,
    P: WinnowParser<I, PO>,
    S: WinnowParser<I, SO>,
{
    separated(N, item, sep)
        .map(|x: Vec<_>| {
            x.try_into()
                .unwrap_or_else(|_| unreachable!("Parser should take care of length"))
        })
        .trace("separated_array")
}

/// winnow::combinator::repeat but exact sized
pub fn repeat_array<const N: usize, I, O>(
    parser: impl WinnowParser<I, O>,
) -> impl WinnowParser<I, [O; N]>
where
    I: Stream,
{
    repeat(N, parser)
        .map(|x: Vec<_>| {
            x.try_into()
                .unwrap_or_else(|_| unreachable!("Parser should take care of length"))
        })
        .trace("repeat_array")
}

/// tail .trace()
pub trait TraceHelper<I, O, E> {
    fn trace(self, name: impl Display) -> impl Parser<I, O, E>;
}

impl<P, I, O, E> TraceHelper<I, O, E> for P
where
    I: Stream,
    E: ParserError<I>,
    P: Parser<I, O, E>,
{
    fn trace(self, name: impl Display) -> impl Parser<I, O, E> {
        trace(name, self)
    }
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use super::unquoted;

    #[test]
    fn test_unquoted() {
        let input = "hello";
        let x = unquoted().parse(input).unwrap();
        assert_eq!(x, "hello");
    }
}
