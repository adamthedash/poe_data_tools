use std::{
    collections::HashMap,
    hash::{BuildHasher, Hasher},
    path::{Path, PathBuf},
};

use anyhow::Context;
use anyhow::Result;
use bytes::Bytes;
use url::Url;

use crate::{
    bundle::{fetch_bundle_content, load_bundle_content},
    bundle_index::{fetch_index_file, load_index_file, BundleIndex},
    hasher::BuildMurmurHash64A,
    path::parse_paths,
};

pub struct FS {
    index: BundleIndex,
    lut: HashMap<u64, usize>,
    steam_folder: Option<PathBuf>,
    base_url: Option<Url>,
    cache_dir: Option<PathBuf>,
}

pub fn from_steam(steam_folder: PathBuf) -> Result<FS> {
    let index_path = steam_folder.as_path().join("Bundles2/_.index.bin");
    let index = load_index_file(&index_path).context("Failed to load bundle index")?;
    Ok(FS {
        index,
        lut: HashMap::new(),
        steam_folder: Some(steam_folder.clone()),
        base_url: None,
        cache_dir: None,
    })
}
pub fn from_cdn(base_url: &Url, cache_dir: &Path) -> Result<FS> {
    let index = fetch_index_file(
        base_url,
        cache_dir,
        PathBuf::from("Bundles2/_.index.bin").as_ref(),
    )
    .context("Failed to load bundle index")?;

    Ok(FS {
        index,
        lut: HashMap::new(),
        steam_folder: None,
        base_url: Some(base_url.clone()),
        cache_dir: Some(cache_dir.to_path_buf()),
    })
}

impl FS {
    pub fn list(&self) -> Vec<String> {
        let mut paths = Vec::new();
        // Loop over each folder
        self.index.paths.iter().for_each(|pr| {
            let parsed = parse_paths(&self.index.path_rep_bundle, pr);
            paths.append(&mut parsed.get_paths());
        });
        paths
    }
    pub fn read(&mut self, path: &str) -> Result<Bytes> {
        if self.lut.is_empty() {
            self.lut = self
                .index
                .files
                .iter()
                .enumerate()
                .map(|(i, f)| (f.hash, i))
                .collect();
        }

        let hash_builder = BuildMurmurHash64A { seed: 0x1337b33f };
        let mut hasher = hash_builder.build_hasher();
        hasher.write(path.to_lowercase().as_bytes());
        let hash = hasher.finish();

        let index = self
            .lut
            .get(&hash)
            .with_context(|| format!("Path not found in index: {}", path))?;
        let file = &self.index.files[*index];

        let bundle = if let Some(steam_folder) = &self.steam_folder {
            let bundle_path = steam_folder.join(format!(
                "Bundles2/{}.bundle.bin",
                self.index.bundles[file.bundle_index as usize].name
            ));
            load_bundle_content(&bundle_path)
                .with_context(|| format!("Failed to load bundle file: {:?}", bundle_path))?
        } else {
            let bundle_path = PathBuf::from(format!(
                "Bundles2/{}.bundle.bin",
                self.index.bundles[file.bundle_index as usize].name
            ));
            fetch_bundle_content(
                self.base_url.as_ref().unwrap(),
                self.cache_dir.as_ref().unwrap(),
                &bundle_path,
            )
            .with_context(|| format!("Failed to fetch bundle file: {:?}", bundle_path))?
        };

        // Pull out the file's contents
        let content = bundle.read_range(file.offset as usize, file.size as usize);
        Ok(content)
    }
}
