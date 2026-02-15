use std::fmt::Display;

use winnow::{
    Parser,
    ascii::dec_uint,
    combinator::{delimited, preceded, separated, trace},
    error::{ContextError, ParserError},
    stream::Stream,
    token::{literal, take_until, take_while},
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

pub fn quoted_str(input: &mut &str) -> winnow::Result<String> {
    delimited('"', take_until(0.., '"'), '"')
        .map(String::from)
        .trace("quoted_str")
        .parse_next(input)
}

pub fn single_quoted_str(input: &mut &str) -> winnow::Result<String> {
    delimited('\'', take_until(0.., '\''), '\'')
        .map(String::from)
        .trace("single_quoted_str")
        .parse_next(input)
}

pub fn unquoted_str(input: &mut &str) -> winnow::Result<String> {
    take_while(1.., |c: char| !c.is_whitespace())
        .map(String::from)
        .trace("unquoted_str")
        .parse_next(input)
}

/// Filename with the provided extension in a "quoted_string"
pub fn filename<'a>(extension: &str) -> impl WinnowParser<&'a str, String> {
    let ext = format!(".{extension}");

    // TODO: to_string after verify passes instead of before
    delimited('"', take_until(0.., '"'), '"')
        .verify(move |s: &str| s.ends_with(&ext))
        .map(String::from)
        .trace("filename")
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
