use std::{
    collections::HashMap,
    hash::{BuildHasher, Hasher},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use iterators_extended::bucket::Bucket;
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

impl FS {
    /// Initialise a file system over a steam folder
    pub fn from_steam(steam_folder: PathBuf) -> Result<FS> {
        let index_path = steam_folder.as_path().join("Bundles2/_.index.bin");
        let index = load_index_file(&index_path).context("Failed to load bundle index")?;

        let lut = index
            .files
            .iter()
            .enumerate()
            .map(|(i, f)| (f.hash, i))
            .collect();

        Ok(FS {
            index,
            lut,
            steam_folder: Some(steam_folder.clone()),
            base_url: None,
            cache_dir: None,
        })
    }

    /// Initialise a file system using the CDN background
    pub fn from_cdn(base_url: &Url, cache_dir: &Path) -> Result<FS> {
        let index = fetch_index_file(
            base_url,
            cache_dir,
            PathBuf::from("Bundles2/_.index.bin").as_ref(),
        )
        .context("Failed to load bundle index")?;

        let lut = index
            .files
            .iter()
            .enumerate()
            .map(|(i, f)| (f.hash, i))
            .collect();

        Ok(FS {
            index,
            lut,
            steam_folder: None,
            base_url: Some(base_url.clone()),
            cache_dir: Some(cache_dir.to_path_buf()),
        })
    }

    /// Lists all paths in the index
    pub fn list(&self) -> impl Iterator<Item = String> + '_ {
        self.index
            .paths
            .iter()
            .flat_map(|p| parse_paths(&self.index.path_rep_bundle, p).get_paths())
    }

    /// Read many files at once, optimising batch loads. Does not preserve order of paths given.
    pub fn batch_read<'a>(
        &'a self,
        paths: &'a [&str],
    ) -> impl Iterator<Item = Result<(&'a str, Bytes), (&'a str, anyhow::Error)>> {
        // Get FileInfo's
        let hash_builder = BuildMurmurHash64A { seed: 0x1337b33f };
        let (fileinfos, errors) = paths
            .iter()
            .map(|&path| {
                // Compute hash
                let mut hasher = hash_builder.build_hasher();
                hasher.write(path.to_lowercase().as_bytes());
                let hash = hasher.finish();

                // Look up the file info for this file
                let fileinfo = self
                    .lut
                    .get(&hash)
                    .map(|i| &self.index.files[*i])
                    .with_context(|| format!("Path not found in index: {}", path))
                    .map(|f| (path, f))
                    .map_err(|e| (path, e));

                fileinfo
            })
            .bucket_result();

        // Batch them into their bundles
        let fileinfos =
            fileinfos
                .into_iter()
                .fold(HashMap::<_, Vec<_>>::new(), |mut acc, (path, fileinfo)| {
                    acc.entry(fileinfo.bundle_index)
                        .or_default()
                        .push((path, fileinfo));

                    acc
                });

        // Process files bundle-wise
        let file_contents = fileinfos.into_iter().flat_map(|(bundle_index, files)| {
            // Load the bundle
            let bundle_path = format!(
                "Bundles2/{}.bundle.bin",
                self.index.bundles[bundle_index as usize].name
            );
            let bundle = if let Some(steam_folder) = &self.steam_folder {
                let bundle_path = steam_folder.join(bundle_path);
                load_bundle_content(&bundle_path)
                    .with_context(|| format!("Failed to load bundle file: {:?}", bundle_path))
            } else {
                let bundle_path = PathBuf::from(bundle_path);
                fetch_bundle_content(
                    self.base_url.as_ref().unwrap(),
                    self.cache_dir.as_ref().unwrap(),
                    &bundle_path,
                )
                .with_context(|| format!("Failed to fetch bundle file: {:?}", bundle_path))
            };

            // Read the file contents - todo: see if we can do this lazily instead of
            // collecting all files within a bundle at once
            let contents: Vec<_> = match bundle {
                Ok(b) => files
                    .into_iter()
                    .map(|(path, file)| {
                        Ok((path, b.read_range(file.offset as usize, file.size as usize)))
                    })
                    .collect(),
                Err(e) => files
                    .into_iter()
                    .map(|(path, _)| Err((path, anyhow!("{:?}", e))))
                    .collect(),
            };

            contents
        });

        // Add on previous errors
        errors.into_iter().map(Err).chain(file_contents)
    }

    pub fn read(&self, path: &str) -> Result<Bytes> {
        // Compute the hash of this file path
        let hash_builder = BuildMurmurHash64A { seed: 0x1337b33f };
        let mut hasher = hash_builder.build_hasher();
        hasher.write(path.to_lowercase().as_bytes());
        let hash = hasher.finish();

        // Look up the file info for this file
        let index = self
            .lut
            .get(&hash)
            .with_context(|| format!("Path not found in index: {}", path))?;
        let file = &self.index.files[*index];

        // Load the bundle
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
