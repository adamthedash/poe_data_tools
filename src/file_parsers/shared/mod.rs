pub mod nom;
pub mod winnow;

use anyhow::{Context, bail};
pub use nom::*;
use regex::Regex;

/// Parse the bytes of a UTF-16 file with BOM
/// https://en.wikipedia.org/wiki/Byte_order_mark#UTF-16
pub fn utf16_bom_to_string(contents: &[u8]) -> anyhow::Result<String> {
    let (first, rest) = contents
        .split_at_checked(2)
        .context("Not enough bytes for BOM")?;

    let parse_utf16 = match first {
        [0xff, 0xfe] => String::from_utf16le,
        [0xfe, 0xff] => String::from_utf16be,
        bytes => bail!("Invalid BOM found: {:?}", bytes),
    };

    parse_utf16(rest).context("Failed to parse contents as UTF-16 string")
}

/// Remove trailing commas so serde can parse it
pub fn remove_trailing(contents: &str) -> String {
    let re = Regex::new(r",\s*([\]}\)])").unwrap();

    let contents = re.replace_all(contents, "$1");

    contents.to_string()
}
