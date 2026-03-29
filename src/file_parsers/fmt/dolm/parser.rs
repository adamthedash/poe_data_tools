use super::types::*;
use crate::file_parsers::shared::winnow::WinnowParser;

pub fn dolm<'a>() -> impl WinnowParser<&'a [u8], Dolm> {
    //
}
