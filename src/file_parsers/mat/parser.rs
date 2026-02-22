use anyhow::Context;

use super::types::*;
use crate::file_parsers::shared::remove_trailing;

pub fn parse_mat_str(contents: &str) -> anyhow::Result<MATFile> {
    let contents = remove_trailing(contents);
    let contents = contents.trim();

    serde_json::from_str(contents).context("Failed to parse file")
}
