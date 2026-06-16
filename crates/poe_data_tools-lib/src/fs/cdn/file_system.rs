use std::{
    borrow::Cow,
    collections::HashMap,
    hash::{BuildHasher, Hasher},
    path::Path,
    sync::Arc,
};

use bytes::Bytes;
use futures::StreamExt;
use iterators_extended::bucket::Bucket;
use url::Url;

use super::CDNLoader;
use crate::{
    file_parsers::{
        FileParser,
        bundle::{BundleParser, types::BundleFile},
        bundle_index::{BundleIndexParser, types::BundleIndexFile},
    },
    fs::{FileSystem, Result, error::Error as FSError},
    hasher::murmur64a::BuildMurmurHash64A,
    path::parse_paths,
};

const HASHER: BuildMurmurHash64A = BuildMurmurHash64A { seed: 0x1337b33f };

/// File system backed by GGG's CDN + local cache
pub struct CDNFS {
    /// Bundle loader
    cdn_loader: Arc<CDNLoader>,
    index: BundleIndexFile,
    /// File hash -> index lookup
    lut: HashMap<u64, usize>,
}

impl CDNFS {
    /// Create a new filesystem backed by the provided CDN and cache location
    pub fn new(base_url: &Url, cache_dir: &Path) -> Result<Self> {
        let cdn_loader = CDNLoader::new(
            base_url,
            cache_dir.to_str().ok_or_else(|| {
                FSError::InvalidConfig(format!("invalid cache path: {cache_dir:?}"))
            })?,
        )?;

        let index = fetch_index_file(&cdn_loader, Path::new("Bundles2/_.index.bin"))?;

        let lut = index
            .files
            .iter()
            .enumerate()
            .map(|(i, f)| (f.hash, i))
            .collect();

        Ok(Self {
            cdn_loader: Arc::new(cdn_loader),
            index,
            lut,
        })
    }
}

impl FileSystem for CDNFS {
    fn list(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(
            self.index
                .paths
                .iter()
                .flat_map(|p| parse_paths(&self.index.path_rep_bundle, p).get_paths()),
        )
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
            .ok_or_else(|| FSError::FileNotFound(path.to_owned()))?;
        let file = &self.index.files[*file_index];

        // Load the bundle
        let bundle_path = format!(
            "Bundles2/{}.bundle.bin",
            self.index.bundles[file.bundle_index as usize].name
        );
        let bundle = fetch_bundle_content(&self.cdn_loader, Path::new(&bundle_path))?;

        // Pull out the file's contents
        bundle.read_range(file.offset as usize, file.size as usize)
    }

    fn batch_read<'a>(
        &'a self,
        paths: &'a [impl AsRef<str>],
    ) -> Box<dyn Iterator<Item = (Cow<'a, str>, Result<Bytes>)> + 'a> {
        // Get FileInfo's
        let (fileinfos, errors) = paths
            .iter()
            .map(|path| {
                let path = path.as_ref().to_owned();
                // Compute hash
                let mut hasher = HASHER.build_hasher();
                hasher.write(path.to_lowercase().as_bytes());
                let hash = hasher.finish();

                // Look up the file info for this file
                match self.lut.get(&hash).map(|i| self.index.files[*i].clone()) {
                    Some(f) => Ok((path, f)),
                    None => {
                        let e = FSError::FileNotFound(path.clone());
                        Err((path, Err(e)))
                    }
                }
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

        // Prepare async tasks
        let file_infos = fileinfos
            .into_iter()
            .map(|(bundle_index, files)| {
                let bundle_path = format!(
                    "Bundles2/{}.bundle.bin",
                    self.index.bundles[bundle_index as usize].name
                );

                (bundle_path, files)
            })
            .map(|(bundle_path, files)| {
                let cdn_loader = Arc::clone(&self.cdn_loader);
                async move {
                    // Load the bundle
                    let res = cdn_loader.load_async(Path::new(&bundle_path)).await;

                    (bundle_path, res, files)
                }
            })
            .collect::<Vec<_>>();

        // Spin up async from here
        const CONCURRENCY: usize = 16;
        let (tx, rx) = std::sync::mpsc::sync_channel(CONCURRENCY);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create async runtime");

            let sender = futures::stream::iter(file_infos)
                .buffer_unordered(CONCURRENCY)
                .for_each(|v| async {
                    tx.send(v).expect("failed to send message");
                });

            rt.block_on(sender);
        });

        let file_contents = rx.into_iter().flat_map(|(_bundle_path, bundle, files)| {
            let bundle = bundle.and_then(|bytes| {
                BundleParser
                    .parse(&bytes)
                    .as_anyhow()
                    .map_err(|e| FSError::Parse(Arc::new(e)))
            });

            let contents: Box<dyn Iterator<Item = _>> = match bundle {
                Ok(b) => Box::new(files.into_iter().map(move |(path, file)| {
                    (path, b.read_range(file.offset as usize, file.size as usize))
                })),
                Err(e) => Box::new(
                    files
                        .into_iter()
                        .map(move |(path, _)| (path, Err(e.clone()))),
                ),
            };

            contents
        });

        // Add on previous errors
        Box::new(
            errors
                .into_iter()
                .chain(file_contents)
                .map(|(path, r)| (Cow::Owned(path), r)),
        )
    }
}

/// Fetch an index file from the CDN (or cache)
fn fetch_index_file(cdn_loader: &CDNLoader, path: &Path) -> Result<BundleIndexFile> {
    let index_content = fetch_bundle_content(cdn_loader, path)?.read_all()?;

    BundleIndexParser
        .parse(&index_content)
        .as_anyhow()
        .map_err(|e| FSError::Parse(Arc::new(e)))
}

// Fetch a bundle file from the CDN (or cache)
fn fetch_bundle_content(cdn_loader: &CDNLoader, path: &Path) -> Result<BundleFile> {
    let bundle_content = cdn_loader.load(path)?;

    BundleParser
        .parse(&bundle_content)
        .as_anyhow()
        .map_err(|e| FSError::Parse(Arc::new(e)))
}
