use std::path::Path;

use anyhow::{Context, Result, anyhow};
use url::Url;

use crate::{
    bundle::{fetch_bundle_content, load_bundle_content},
    file_parsers::{
        FileParser,
        bundle_index::{BundleIndexParser, types::BundleIndexFile},
    },
};

/// Load an index file from disk
pub fn load_index_file(path: &Path) -> Result<BundleIndexFile> {
    let index_content = load_bundle_content(path)
        .context("Failed to read bundle index")?
        .read_all();

    BundleIndexParser
        .parse(&index_content)
        .map_err(|_| anyhow!("Failed to parse bundle as index"))
}

/// Fetch an index file from the CDN (or cache)
pub fn fetch_index_file(base_url: &Url, cache_dir: &Path, path: &Path) -> Result<BundleIndexFile> {
    let index_content = fetch_bundle_content(base_url, cache_dir, path)
        .context("Failed to fetch bundle index")?
        .read_all();

    BundleIndexParser
        .parse(&index_content)
        .map_err(|_| anyhow!("Failed to parse bundle as index"))
}
