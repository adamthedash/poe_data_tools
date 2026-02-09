use std::{fmt::Debug, num::ParseIntError};

#[derive(Debug)]
pub enum Error {
    Incomplete,
    // TODO: Better parse error types
    // TODO: Better tracing for inner errors
    ParseError(Box<dyn std::error::Error>),
    // For all-consuming parsers
    DataRemaining,
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::ParseError(value.into())
    }
}

impl<E> From<nom::Err<E>> for Error
where
    E: Debug,
{
    fn from(value: nom::Err<E>) -> Self {
        match value {
            nom::Err::Incomplete(_) => Error::Incomplete,
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                Error::ParseError(format!("{:?}", e).into())
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Signature of a line parser. Takes in lines as a slice of strings,
/// Returns the parsed item and remaining unused lines
pub trait MultilineParser<'a, T> = FnMut(&'a [&'a str]) -> Result<(&'a [&'a str], T)>;

pub trait SingleLineParser<'a, T> = FnMut(&'a str) -> Result<(&'a str, T)>;

/// Count on single line followed by N applications of the inner parser
pub fn length_prefixed<'a, T>(
    mut item_parser: impl MultilineParser<'a, T>,
) -> impl MultilineParser<'a, Vec<T>> {
    move |lines| {
        let Some((count, rest)) = lines.split_first() else {
            return Err(Error::Incomplete);
        };

        let count = count.parse::<usize>()?;

        let (rest, items) =
            (0..count).try_fold((rest, vec![]), |(rest, mut items), _i| -> Result<_> {
                let (rest, item) = item_parser(rest)?;

                items.push(item);

                Ok((rest, items))
            })?;

        Ok((rest, items))
    }
}

pub fn repeated<'a, T>(
    mut item_parser: impl MultilineParser<'a, T>,
    count: usize,
) -> impl MultilineParser<'a, Vec<T>> {
    move |lines| {
        (0..count).try_fold((lines, vec![]), |(rest, mut items), _i| -> Result<_> {
            let (rest, item) = item_parser(rest)?;

            items.push(item);

            Ok((rest, items))
        })
    }
}

/// Apply the inner parser until the sentinel line is found
pub fn terminated<'a, T>(
    mut item_parser: impl MultilineParser<'a, T>,
    sentinel: &'a str,
) -> impl MultilineParser<'a, Vec<T>> {
    move |mut lines| {
        let mut items = vec![];
        loop {
            // Check if we're at the end
            match lines.first() {
                None => return Err(Error::Incomplete),
                Some(l) if *l == sentinel => return Ok((&lines[1..], items)),
                Some(_) => {}
            };

            // Apply inner
            let (rest, item) = item_parser(lines)?;
            items.push(item);
            lines = rest;
        }
    }
}

/// Apply the inner parser until we run out of input
pub fn take_forever<'a, T>(
    mut item_parser: impl MultilineParser<'a, T>,
) -> impl MultilineParser<'a, Vec<T>> {
    move |mut lines| {
        let mut items = vec![];
        loop {
            // Check if there's any more lines
            if lines.is_empty() {
                return Ok((lines, items));
            }

            // Apply inner
            let (rest, item) = item_parser(lines)?;
            items.push(item);
            lines = rest;
        }
    }
}

/// Apply the inner parser as many as we can, stopping at the first failure or when input is
/// exhausted
pub fn take_many<'a, T>(
    mut item_parser: impl MultilineParser<'a, T>,
) -> impl MultilineParser<'a, Vec<T>> {
    move |mut lines| {
        let mut items = vec![];
        while let Ok((rest, item)) = item_parser(lines) {
            items.push(item);
            lines = rest;
        }

        Ok((lines, items))
    }
}

/// Try to apply the inner parser. If it fails, return the full input untouched
pub fn optional<'a, T>(
    mut item_parser: impl MultilineParser<'a, T>,
) -> impl MultilineParser<'a, Option<T>> {
    move |lines| {
        if let Ok((lines, item)) = item_parser(lines) {
            Ok((lines, Some(item)))
        } else {
            Ok((lines, None))
        }
    }
}

/// Adapts a single-line parser to a multi-line one
/// Inner parser must consume the entire line
pub fn single_line<'a, T>(
    mut line_parser: impl SingleLineParser<'a, T>,
) -> impl MultilineParser<'a, T> {
    move |lines| {
        let Some((first, rest)) = lines.split_first() else {
            return Err(Error::Incomplete);
        };

        let (rest_inner, item) = line_parser(first)?;
        if !rest_inner.is_empty() {
            return Err(Error::DataRemaining);
        }

        Ok((rest, item))
    }
}

/// Adapts a nom &str parser into a SingleLineParser
pub fn nom_adapter<'a, T>(
    mut nom_parser: impl nom::Parser<&'a str, T, nom::error::Error<&'a str>>,
) -> impl SingleLineParser<'a, T> {
    move |line| nom_parser.parse(line).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use nom::bytes::complete::tag;

    use super::{Error, length_prefixed, nom_adapter, single_line, terminated};
    use crate::file_parsers::line_parser::{optional, take_forever, take_many};

    #[test]
    fn test_length_prefixed_good() {
        let lines = ["2", "a", "b", "c"];

        let item_parser = |line| Ok(("", line));
        let line_parser = single_line(item_parser);
        let mut parser = length_prefixed(line_parser);

        let (rest, parsed) = parser(&lines).unwrap();

        assert_eq!(&parsed, &["a", "b"]);
        assert_eq!(rest, &["c"]);
    }

    #[test]
    fn test_length_prefixed_incomplete() {
        let lines = [];

        let item_parser = |line| Ok(("", line));
        let line_parser = single_line(item_parser);
        let mut parser = length_prefixed(line_parser);

        let err = parser(&lines).unwrap_err();

        assert_matches!(err, Error::Incomplete);
    }

    #[test]
    fn test_length_prefixed_bad_count() {
        let lines = ["x", "a", "b", "c"];

        let item_parser = |line| Ok(("", line));
        let line_parser = single_line(item_parser);
        let mut parser = length_prefixed(line_parser);

        let err = parser(&lines).unwrap_err();

        assert_matches!(err, Error::ParseError(..));
    }

    #[test]
    fn test_length_prefixed_bad_inner() {
        let lines = ["2", "a"];

        let item_parser = |line| Ok(("", line));
        let line_parser = single_line(item_parser);
        let mut parser = length_prefixed(line_parser);

        let err = parser(&lines).unwrap_err();

        assert_matches!(err, Error::Incomplete);
    }

    #[test]
    fn test_terminated_good() {
        let lines = ["a", "b", "-1", "c"];

        let item_parser = |line| Ok(("", line));
        let line_parser = single_line(item_parser);
        let mut parser = terminated(line_parser, "-1");

        let (rest, items) = parser(&lines).unwrap();
        assert_eq!(&items, &["a", "b"]);
        assert_eq!(rest, &["c"]);
    }

    #[test]
    fn test_terminated_no_sentinel() {
        let lines = ["a", "b", "c"];

        let item_parser = |line| Ok(("", line));
        let line_parser = single_line(item_parser);
        let mut parser = terminated(line_parser, "-1");

        let err = parser(&lines).unwrap_err();
        assert_matches!(err, Error::Incomplete);
    }

    #[test]
    fn test_nom_adapter_good() {
        use nom::{character::complete, sequence::separated_pair};

        let line = "123 456";

        let nom_parser = separated_pair(complete::u32, complete::char(' '), complete::u32);
        let mut parser = nom_adapter(nom_parser);

        let (rest, digits) = parser(line).unwrap();
        assert_eq!(rest, "");
        assert_eq!(digits, (123, 456));
    }

    #[test]
    fn test_nom_adapter_incomplete() {
        use nom::{character::complete, sequence::separated_pair};

        let line = "123 ";

        let nom_parser = separated_pair(complete::u32, complete::char(' '), complete::u32);
        let mut parser = nom_adapter(nom_parser);

        let err = parser(line).unwrap_err();
        assert_matches!(err, Error::ParseError(..));
    }

    #[test]
    fn test_take_forever_good() {
        let lines = ["a", "a", "a"];

        let line_parser = single_line(nom_adapter(tag("a")));
        let mut parser = take_forever(line_parser);

        let (rest, digits) = parser(&lines).unwrap();
        assert!(rest.is_empty());
        assert_eq!(digits, ["a", "a", "a"]);
    }

    #[test]
    fn test_take_forever_bad() {
        let lines = ["a", "a", "b"];

        let line_parser = single_line(nom_adapter(tag("a")));
        let mut parser = take_forever(line_parser);

        let err = parser(&lines).unwrap_err();
        assert_matches!(err, Error::ParseError(..));
    }

    #[test]
    fn test_optional_some() {
        let lines = ["a", "b"];

        let line_parser = single_line(nom_adapter(tag("a")));
        let mut parser = optional(line_parser);

        let (lines, item) = parser(&lines).unwrap();
        assert_eq!(item, Some("a"));
        assert_eq!(lines, &["b"]);
    }

    #[test]
    fn test_optional_none() {
        let lines = ["b", "b"];

        let line_parser = single_line(nom_adapter(tag("a")));
        let mut parser = optional(line_parser);

        let (lines, item) = parser(&lines).unwrap();
        assert_eq!(item, None);
        assert_eq!(lines, &["b", "b"]);
    }

    #[test]
    fn test_take_many_end_input() {
        let lines = ["a", "a"];

        let line_parser = single_line(nom_adapter(tag("a")));
        let mut parser = take_many(line_parser);

        let (lines, item) = parser(&lines).unwrap();
        assert_eq!(item[..], ["a", "a"]);
        assert!(lines.is_empty())
    }

    #[test]
    fn test_take_many_end_other() {
        let lines = ["a", "a", "b"];

        let line_parser = single_line(nom_adapter(tag("a")));
        let mut parser = take_many(line_parser);

        let (lines, item) = parser(&lines).unwrap();
        assert_eq!(item[..], ["a", "a"]);
        assert_eq!(lines, &["b"]);
    }

    #[test]
    fn test_take_many_none() {
        let lines = ["b"];

        let line_parser = single_line(nom_adapter(tag("a")));
        let mut parser = take_many(line_parser);

        let (lines, item) = parser(&lines).unwrap();
        assert!(item.is_empty());
        assert_eq!(lines, &["b"]);
    }
}
