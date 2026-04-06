use std::{
    collections::HashMap,
    hash::{BuildHasher, Hasher},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use bytes::Bytes;
use iterators_extended::bucket::Bucket;
use url::Url;

use super::FileSystem;
use crate::{
    bundle::fetch_bundle_content, bundle_index::fetch_index_file,
    file_parsers::bundle_index::types::BundleIndexFile, hasher::murmur64a::BuildMurmurHash64A,
    path::parse_paths,
};

const HASHER: BuildMurmurHash64A = BuildMurmurHash64A { seed: 0x1337b33f };

pub struct CDNFS {
    index: BundleIndexFile,
    lut: HashMap<u64, usize>,
    base_url: Url,
    cache_dir: PathBuf,
}

impl CDNFS {
    pub fn new(base_url: &Url, cache_dir: &Path) -> Result<Self> {
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

        Ok(Self {
            index,
            lut,
            base_url: base_url.clone(),
            cache_dir: cache_dir.to_path_buf(),
        })
    }
}

impl FileSystem for CDNFS {
    /// Lists all paths in the index
    fn list(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(
            self.index
                .paths
                .iter()
                .flat_map(|p| parse_paths(&self.index.path_rep_bundle, p).get_paths()),
        )
    }

    /// Read many files at once, optimising batch loads. Does not preserve order of paths given.
    fn batch_read<'a>(
        &'a self,
        paths: &'a [impl AsRef<str>],
    ) -> Box<dyn Iterator<Item = Result<(&'a str, Bytes), (&'a str, anyhow::Error)>> + 'a> {
        // Get FileInfo's
        let (fileinfos, errors) = paths
            .iter()
            .map(|path| {
                let path = path.as_ref();
                // Compute hash
                let mut hasher = HASHER.build_hasher();
                hasher.write(path.to_lowercase().as_bytes());
                let hash = hasher.finish();

                // Look up the file info for this file
                self.lut
                    .get(&hash)
                    .map(|i| &self.index.files[*i])
                    .with_context(|| format!("Path not found in index: {}", path))
                    .map(|f| (path, f))
                    .map_err(|e| (path, e))
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
            let bundle =
                fetch_bundle_content(&self.base_url, &self.cache_dir, Path::new(&bundle_path))
                    .with_context(|| format!("Failed to fetch bundle file: {:?}", bundle_path));

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
        Box::new(errors.into_iter().map(Err).chain(file_contents))
    }

    fn read(&self, path: &str) -> Result<Bytes> {
        // Compute the hash of this file path
        let mut hasher = HASHER.build_hasher();
        hasher.write(path.to_lowercase().as_bytes());
        let hash = hasher.finish();

        // Look up the file info for this file
        let file_index = self
            .lut
            .get(&hash)
            .with_context(|| format!("Path not found in index: {}", path))?;
        let file = &self.index.files[*file_index];

        // Load the bundle
        let bundle_path = format!(
            "Bundles2/{}.bundle.bin",
            self.index.bundles[file.bundle_index as usize].name
        );
        let bundle = fetch_bundle_content(&self.base_url, &self.cache_dir, Path::new(&bundle_path))
            .with_context(|| format!("Failed to fetch bundle file: {:?}", bundle_path))?;

        // Pull out the file's contents
        let content = bundle.read_range(file.offset as usize, file.size as usize);
        Ok(content)
    }
}
