use std::{
    collections::HashMap,
    hash::{BuildHasher, Hasher},
    path::{Path, PathBuf},
};

use bytes::Bytes;

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
    patch: Option<String>,
    cache_dir: Option<PathBuf>,
}

pub fn from_steam(steam_folder: PathBuf) -> FS {
    let index_path = steam_folder.as_path().join("Bundles2/_.index.bin");
    let index = load_index_file(&index_path);
    FS {
        index,
        lut: HashMap::new(),
        steam_folder: Some(steam_folder.clone()),
        patch: None,
        cache_dir: None,
    }
}
pub fn from_cdn(cache_dir: &Path, patch: &str) -> FS {
    let index = fetch_index_file(
        patch,
        cache_dir,
        PathBuf::from("Bundles2/_.index.bin").as_ref(),
    );
    FS {
        index,
        lut: HashMap::new(),
        steam_folder: None,
        patch: Some(patch.to_string()),
        cache_dir: Some(cache_dir.to_path_buf()),
    }
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
    pub fn read(&mut self, path: String) -> anyhow::Result<Bytes> {
        if self.lut.is_empty() {
            eprintln!("Building lookup-table");
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
            .unwrap_or_else(|| panic!("Path not found in index: {}", path));
        let file = &self.index.files[*index];

        // FIXME: This is expensive and gets repeated even for files in bundles
        // we've already seen
        //
        // I need to either optimize it heavily for random reads, or extract the
        // entire bundle and cache all the files so when we are after a
        // different file in the bundle we can skip this
        let bundle = match self.steam_folder {
            Some(ref steam_folder) => {
                let bundle_path = steam_folder.join(format!(
                    "Bundles2/{}.bundle.bin",
                    self.index.bundles[file.bundle_index as usize].name
                ));
                load_bundle_content(bundle_path.as_ref())
            }
            None => fetch_bundle_content(
                self.patch.as_ref().unwrap(),
                self.cache_dir.as_ref().unwrap(),
                PathBuf::from(format!(
                    "Bundles2/{}.bundle.bin",
                    self.index.bundles[file.bundle_index as usize].name
                ))
                .as_ref(),
            ),
        };

        println!("Extracting: {}", path);

        // Pull out the file's contents
        Ok(bundle.slice(file.offset as usize..file.offset as usize + file.size as usize))
    }
}
