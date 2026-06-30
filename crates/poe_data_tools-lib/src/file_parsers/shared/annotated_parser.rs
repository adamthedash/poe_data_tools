use annotated_parser::{combinators::TakeTillExc, parsers::TakeArray, prelude::*};

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
