use winnow::{
    Parser,
    combinator::{impls::Context, trace},
    error::{ContextError, Result, StrContext},
    stream::Stream,
};

/// Winnow parser over `&[I]` input
pub trait SliceParser<'a, I, O, E> = Parser<&'a [I], O, E> where I: 'a;

pub struct Lift<P> {
    inner: P,
}

impl<P, I, O> Lift<Context<P, I, O, ContextError, StrContext>>
where
    I: Stream,
    P: Parser<I, O, ContextError>,
{
    fn new(parser: P) -> Self {
        let inner = parser.context(StrContext::Label("inner"));
        Lift { inner }
    }
}

impl<P, I, S, O> Parser<S, O, ContextError> for Lift<P>
where
    I: Stream,
    S: Stream<Token = I>,
    P: Parser<I, O, ContextError>,
{
    fn parse_next(&mut self, input: &mut S) -> Result<O> {
        let checkpoint = input.checkpoint();

        let Some(mut token) = input.next_token() else {
            let mut context = ContextError::new();
            context.extend([StrContext::Label("outer")]);
            return Err(context);
        };

        // TODO: Does this ensure input is fully consumed?
        //      Also do we need to add more context here?
        let result = self.inner.parse_next(&mut token);
        if result.is_err() {
            // Reset input back to where it was before
            input.reset(&checkpoint);
        }

        result
    }
}

/// "Lifts" the parser up one level, allowing it to parse &[I] instead of I
pub fn lift<I, S, O, P>(parser: P) -> impl Parser<S, O, ContextError>
where
    I: Stream,
    S: Stream<Token = I>,
    P: Parser<I, O, ContextError>,
{
    trace("lift", Lift::new(parser))
}

#[cfg(test)]
mod tests {
    use winnow::{Parser, error::StrContext, token::literal};

    use super::{Lift, lift};

    #[test]
    fn test_nested() {
        let input = vec!["as", "a", "b"];
        let mut input = input.as_slice();

        // In-line parser
        let parser = literal("a");

        // Over-line parser
        let mut line_parser = Lift::new(parser);

        let parsed = line_parser.parse_next(&mut input).unwrap();
        assert_eq!(parsed, "a");
        assert_eq!(input, &["a", "b"]);
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<&str> = vec![];
        let mut input = input.as_slice();

        // In-line parser
        let parser = literal("a");

        // Over-line parser
        let mut line_parser = lift(parser);

        let err = line_parser.parse_next(&mut input).unwrap_err();
        let context = err.context().next().unwrap();
        assert_eq!(context, &StrContext::Label("outer"))
    }

    #[test]
    fn test_parse_failure() {
        let input: Vec<&str> = vec!["b", "a"];
        let mut input = input.as_slice();

        let parser = literal("a");
        let mut line_parser = lift(parser);

        let err = line_parser.parse_next(&mut input).unwrap_err();
        let context = err.context().next().unwrap();
        assert_eq!(context, &StrContext::Label("inner"));
        // Input should remain unchanged on failure
        assert_eq!(input, &["b", "a"]);
    }
}
