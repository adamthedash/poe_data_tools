use anyhow::Context;
use regex::Regex;

use super::types::*;

/// Remove trailing commas so serde can parse it
fn remove_trailing(contents: &str) -> String {
    let re = Regex::new(r",\s*([\]}\)])").unwrap();

    let contents = re.replace_all(contents, "$1");

    contents.to_string()
}

pub fn parse_mat_str(contents: &str) -> anyhow::Result<MATFile> {
    let contents = remove_trailing(contents);
    let contents = contents.trim();

    serde_json::from_str(contents).context("Failed to parse file")
}
