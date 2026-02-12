use nom::{
    Err, Input, Mode, OutputMode, PResult, Parser,
    combinator::all_consuming,
    error::{Error, ErrorKind, ParseError},
};

use crate::file_parsers::my_slice::MySlice;

/// Nom parser over `&[I]` input
pub trait SliceParser<'a, I, O, E = Error<MySlice<&'a [I]>>> = Parser<MySlice<&'a [I]>, Output = O, Error = E>
where
    I: 'a,
    E: ParseError<MySlice<&'a [I]>>;

pub struct Lift<P> {
    inner: P,
}

impl<'a, P, I> Parser<MySlice<&'a [I]>> for Lift<P>
where
    I: Input,
    P: Parser<I>,
    P::Error: ParseError<I>,
{
    type Output = P::Output;

    type Error = Error<MySlice<&'a [I]>>;

    fn process<OM: OutputMode>(
        &mut self,
        input: MySlice<&'a [I]>,
    ) -> PResult<OM, MySlice<&'a [I]>, Self::Output, Self::Error> {
        // Handle where there's no more input
        let Some((first, rest)) = input.split_first() else {
            return Err(Err::Error(OM::Error::bind(|| {
                Self::Error::from_error_kind(input, ErrorKind::Eof)
            })));
        };

        // Apply the inner parser
        match self.inner.process::<OM>(first.clone()) {
            Ok((_, item)) => Ok((MySlice(rest), item)),
            // TODO: Figure out a way to bubble up the inner parser error and replace I with
            // &[I]
            Err(_) => Err(Err::Error(OM::Error::bind(|| {
                Self::Error::from_error_kind(input, ErrorKind::Fix)
            }))),
        }
    }
}

/// "Lifts" the parser up one level, allowing it to parse &[I] instead of I
pub fn lift<'a, I, EP, P>(parser: P) -> impl SliceParser<'a, I, P::Output>
where
    I: Input + 'a,
    P: Parser<I, Error = EP>,
    EP: ParseError<I>,
{
    Lift {
        inner: all_consuming(parser),
    }
}

pub trait ToSliceParser<I>: Parser<I> {
    /// "Lifts" the parser up one level, allowing it to parse &[I] instead of I
    fn lift<'a>(self) -> impl SliceParser<'a, I, Self::Output>
    where
        I: 'a;
}

impl<I, P> ToSliceParser<I> for P
where
    I: Input,
    P: Parser<I>,
{
    fn lift<'a>(self) -> impl SliceParser<'a, I, Self::Output, Error<MySlice<&'a [I]>>>
    where
        I: 'a,
    {
        lift(self)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use nom::{
        Parser,
        bytes::tag,
        combinator::all_consuming,
        multi::{count, many1},
    };

    use crate::file_parsers::{lift::ToSliceParser, my_slice::MySlice};

    #[test]
    fn test_nested() {
        let input: MySlice<&[_]> = ["a", "a", "b"].as_slice().into();

        // In-line parser
        let parser = tag("a");
        let parser = all_consuming::<&str, nom::error::Error<_>, _>(parser);

        // Over-line parser
        let line_parser = parser.lift();
        let mut line_parser = many1(line_parser);

        let (rest, parsed) = line_parser.parse(input).unwrap();
        assert_eq!(parsed, ["a", "a"]);
        assert_eq!(rest, ["b"].as_slice().into());
    }

    #[test]
    fn test_nested_bad() {
        let input: MySlice<&[_]> = ["a", "a", "b"].as_slice().into();

        // In-line parser
        let parser = tag("a");
        let parser = all_consuming::<&str, nom::error::Error<_>, _>(parser);

        // Over-line parser
        let line_parser = parser.lift();
        let mut line_parser = count(line_parser, 3);

        let err = line_parser.parse(input).unwrap_err();
        assert_matches!(err, nom::Err::Error(..))
    }
}
