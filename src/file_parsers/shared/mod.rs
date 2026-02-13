pub mod nom;
pub mod winnow;

use anyhow::{Context, bail};
pub use nom::*;

/// Parse the bytes of a UTF-16 file with BOM
/// https://en.wikipedia.org/wiki/Byte_order_mark#UTF-16
pub fn utf16_bom_to_string(contents: &[u8]) -> anyhow::Result<String> {
    let parse_ut16 = match &contents[..2] {
        [0xff, 0xfe] => String::from_utf16le,
        [0xfe, 0xff] => String::from_utf16be,
        bytes => bail!("Invalid BOM found: {:?}", bytes),
    };

    parse_ut16(&contents[2..]).context("Failed to parse contents as UTF-16 string")
}
