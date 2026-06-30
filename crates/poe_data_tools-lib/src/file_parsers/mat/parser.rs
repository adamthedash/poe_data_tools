use super::types::*;
use crate::file_parsers::{
    error::{AsParseError, ParseErrorInner, Result},
    shared::remove_trailing,
};

pub fn parse_mat_str(contents: &str) -> Result<MATFile> {
    let contents = remove_trailing(contents);
    let contents = contents.trim();

    serde_json::from_str(contents)
        .map_err(|e| ParseErrorInner::Other(Box::new(e)))
        .to_parse_error()
}
