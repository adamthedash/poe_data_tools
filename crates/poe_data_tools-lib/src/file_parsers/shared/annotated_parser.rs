use annotated_parser::{
    AnnotationReturn, ParseWithResult, combinators::TakeTillExc, parsers::TakeArray, prelude::*,
};

/// Simpler trait alias for byte parsers
pub trait U8Parser: for<'a> Parser<&'a [u8]> {}
impl<P> U8Parser for P where P: for<'a> Parser<&'a [u8]> {}

/// Simpler trait alias for string parsers
pub trait StrParser: for<'a> Parser<&'a str> {}
impl<P> StrParser for P where P: for<'a> Parser<&'a str> {}

/// For proper input type inference
pub fn take_arr_u8<const N: usize>() -> impl U8Parser<Output = [u8; N]> {
    TakeArray::<N>
}

/// For proper input type inference
fn take_arr_str<const N: usize>() -> impl StrParser<Output = String> {
    TakeArray::<N>
}

pub fn whitespace() -> impl StrParser<Output = String> {
    take_arr_str::<1>()
        .verify(|s| s.chars().next().unwrap().is_ascii_whitespace())
        .many()
        .map(|chars| chars.concat())
        .trace_opaque("whitespace")
}

pub fn quoted() -> impl StrParser<Output = String> {
    TakeTillExc::new("\"")
        .surrounded_by_sym("\"")
        .trace_opaque("quoted")
}

/// Helper function to convert annotated errors into anyhow with proper context
fn anno_to_anyhow(annotation: AnnotationReturn) -> anyhow::Error {
    let AnnotationReturn::Annotated(mut annotation) = annotation else {
        return annotation.into();
    };
    annotation = annotation.to_failure_tree().expect("error path");
    annotation.materialize();

    let mut stack = annotation.failure_path().into_iter();

    let first = stack
        .next()
        .expect("If annotation is a failure then there should be at least 1 entry");

    stack.fold(anyhow::Error::from(first), |err, anno| err.context(anno))
}

pub trait ToAnyhow {
    type Value;

    fn to_anyhow(self) -> anyhow::Result<Self::Value>;
}

impl<T> ToAnyhow for ParseWithResult<T> {
    type Value = T;

    fn to_anyhow(self) -> anyhow::Result<Self::Value> {
        match self {
            Ok((v, _)) => Ok(v),
            Err(a) => Err(anno_to_anyhow(a)),
        }
    }
}
