pub mod annotated_parser;
pub mod lift;
pub mod serialise;
pub mod winnow;

use std::string::FromUtf16Error;

use regex::Regex;

use crate::file_parsers::error::Result;

/// Errors related to parsing UTF-16 encoded bytes with Byte Order Marker
#[derive(Debug, thiserror::Error)]
pub enum BOMError {
    #[error("not enough bytes for BOM")]
    NotEnoughBytes,
    #[error("invalid BOM btyes: {0:?}")]
    InvalidBytes([u8; 2]),
    #[error(transparent)]
    InvalidUTF16(#[from] FromUtf16Error),
}

/// Parse the bytes of a UTF-16 file with BOM
/// https://en.wikipedia.org/wiki/Byte_order_mark#UTF-16
pub fn utf16_bom_to_string(contents: &[u8]) -> Result<String, BOMError> {
    let Some((first, rest)) = contents.split_first_chunk() else {
        return Err(BOMError::NotEnoughBytes);
    };

    let parse_utf16 = match first {
        [0xff, 0xfe] => String::from_utf16le,
        [0xfe, 0xff] => String::from_utf16be,
        bytes => return Err(BOMError::InvalidBytes(*bytes)),
    };

    Ok(parse_utf16(rest)?)
}

/// Remove trailing commas so serde can parse it
pub fn remove_trailing(contents: &str) -> String {
    let re = Regex::new(r",\s*([\]}\)])").unwrap();

    let contents = re.replace_all(contents, "$1");

    contents.to_string()
}
