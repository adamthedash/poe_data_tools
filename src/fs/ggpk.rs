use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    fs::File,
    hash::{BuildHasher, Hasher},
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use anyhow::{Context, anyhow};
use bytes::Bytes;
use iterators_extended::bucket::Bucket;

use crate::{
    file_parsers::{
        FileParser,
        bundle::BundleParser,
        bundle_index::{BundleIndexParser, types::BundleIndexFile},
        ggpk::{
            parser::parse_ggpk,
            types::{Entry, EntryData, GGPKFile},
        },
    },
    fs::FileSystem,
    hasher::murmur64a::BuildMurmurHash64A,
    path::parse_paths,
};

#[derive(Debug, Clone)]
struct FileInfo {
    offset: usize,
    length: usize,
}

/// A file system over the the Content.ggpk file
pub struct GGPKFS {
    file: RefCell<BufReader<File>>,
    index: GGPKFile,
    lut: HashMap<u64, FileInfo>,
}

const HASHER: BuildMurmurHash64A = BuildMurmurHash64A { seed: 0x1337b33f };

/// File info + hash of full file path
fn enumerate_file_info(
    entries: &[Entry],
    prefix: Option<String>,
) -> impl Iterator<Item = (u64, FileInfo)> {
    let prefix = prefix.unwrap_or_default();

    entries
        .iter()
        .flat_map(move |e| -> Box<dyn Iterator<Item = (u64, FileInfo)>> {
            // FIXME: Don't like all the boxing here, also try figure out a stack based prefix
            // rather than allocating the string each time

            let mut name = format!("{prefix}{}", e.name);

            match &e.data {
                EntryData::Dir(items) => {
                    name.push('/');
                    Box::new(enumerate_file_info(items, Some(name)))
                }
                &EntryData::File { offset, length } => {
                    // NOTE: Using our own full path hashes rather than stored MurmurHash2 values from GGPK
                    // as there are duplicate file name hashes that refer to distinct files
                    let hash = HASHER.hash_one(name.to_lowercase().as_bytes());
                    Box::new(std::iter::once((hash, FileInfo { offset, length })))
                }
            }
        })
}

impl GGPKFS {
    pub fn new(ggpk_path: &Path) -> anyhow::Result<Self> {
        let mut file = BufReader::new(File::open(ggpk_path)?);
        let index = parse_ggpk(&mut file)?;

        // Build LUT
        let lut = HashMap::from_iter(enumerate_file_info(&index.entries, None));

        Ok(Self {
            file: RefCell::new(file),
            index,
            lut,
        })
    }

    fn _read(&self, offset: usize, length: usize) -> std::io::Result<Bytes> {
        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start(offset as u64))?;

        let mut buf = vec![0; length];
        file.read_exact(&mut buf)?;

        let buf = Bytes::from(buf);

        Ok(buf)
    }
}

fn enumerate_file_names(entries: &[Entry], prefix: Option<String>) -> impl Iterator<Item = String> {
    let prefix = prefix.unwrap_or_default();

    entries
        .iter()
        .flat_map(move |e| -> Box<dyn Iterator<Item = String>> {
            // FIXME: Don't like all the boxing here, also try figure out a stack based prefix
            // rather than allocating the string each time

            let mut name = format!("{prefix}{}", e.name);

            match &e.data {
                EntryData::Dir(items) => {
                    name.push('/');
                    Box::new(enumerate_file_names(items, Some(name)))
                }
                EntryData::File { .. } => Box::new(std::iter::once(name.to_lowercase())),
            }
        })
}

impl FileSystem for GGPKFS {
    fn list(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(enumerate_file_names(&self.index.entries, None))
    }

    #[allow(clippy::type_complexity)]
    fn batch_read<'a>(
        &'a self,
        paths: &'a [impl AsRef<str>],
    ) -> Box<dyn Iterator<Item = Result<(Cow<'a, str>, Bytes), (Cow<'a, str>, anyhow::Error)>> + 'a>
    {
        // Get FileInfo's
        let (mut fileinfos, errors) = paths
            .iter()
            .map(|path| {
                let path = path.as_ref();
                // Compute hash
                let hash = HASHER.hash_one(path.to_lowercase().as_bytes());

                // Look up the file info for this file
                self.lut
                    .get(&hash)
                    .with_context(|| format!("Path not found in index: {}", path))
                    .map(|f| (path, f))
                    .map_err(|e| (path, e))
            })
            .bucket_result();

        // Order by offset to hopefully get better buffer usage / less seek overhead
        fileinfos.sort_unstable_by_key(|(_, f)| f.offset);

        let file_contents = fileinfos.into_iter().map(|(path, fileinfo)| {
            self._read(fileinfo.offset, fileinfo.length)
                .context("Failed to read file")
                .map(|f| (path, f))
                .map_err(|e| (path, e))
        });

        // Add on previous errors
        Box::new(errors.into_iter().map(Err).chain(file_contents).map(|r| {
            r.map(|(s, b)| (Cow::Borrowed(s), b))
                .map_err(|(s, e)| (Cow::Borrowed(s), e))
        }))
    }

    fn read(&self, path: &str) -> anyhow::Result<Bytes> {
        // Compute the hash of this file path
        let hash = HASHER.hash_one(path.to_lowercase().as_bytes());

        // Look up the file info for this file
        let fileinfo = self
            .lut
            .get(&hash)
            .with_context(|| format!("Path not found in index: {}", path))?;

        // Read the contents
        let buf = self._read(fileinfo.offset, fileinfo.length)?;

        Ok(buf)
    }
}

/// A file system over the bundles within the Content.ggpk file
pub struct GGPKBundleFS {
    ggpk: GGPKFS,
    lut: HashMap<u64, usize>,
    index: BundleIndexFile,
}

impl GGPKBundleFS {
    pub fn new(ggpk_path: &Path) -> anyhow::Result<Self> {
        let ggpk = GGPKFS::new(ggpk_path)?;

        let index_bytes = ggpk
            .read("/Bundles2/_.index.bin")
            .context("Failed to load bundle index from GGPK")?;
        let index_bundle = BundleParser
            .parse(&index_bytes)
            .as_anyhow()
            .context("Failed to parse bundle")?;
        let index = BundleIndexParser
            .parse(&index_bundle.read_all()?)
            .as_anyhow()
            .context("Failed to parse bundle as index")?;

        let lut = index
            .files
            .iter()
            .enumerate()
            .map(|(i, f)| (f.hash, i))
            .collect();

        Ok(Self { ggpk, lut, index })
    }
}

impl FileSystem for GGPKBundleFS {
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
    ) -> Box<dyn Iterator<Item = Result<(Cow<'a, str>, Bytes), (Cow<'a, str>, anyhow::Error)>> + 'a>
    {
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
                "/Bundles2/{}.bundle.bin",
                self.index.bundles[bundle_index as usize].name
            );
            let bundle = self
                .ggpk
                .read(&bundle_path)
                .with_context(|| format!("Failed to load bundle file: {:?}", bundle_path))
                .and_then(|bundle_contents| {
                    BundleParser
                        .parse(&bundle_contents)
                        .as_anyhow()
                        .context("Failed to parse bundle")
                });

            // Read the file contents - todo: see if we can do this lazily instead of
            // collecting all files within a bundle at once
            let contents: Vec<_> = match bundle {
                Ok(b) => files
                    .into_iter()
                    .map(|(path, file)| {
                        b.read_range(file.offset as usize, file.size as usize)
                            .map(|b| (path, b))
                            .map_err(|e| (path, e))
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
        Box::new(errors.into_iter().map(Err).chain(file_contents).map(|r| {
            r.map(|(s, b)| (Cow::Borrowed(s), b))
                .map_err(|(s, e)| (Cow::Borrowed(s), e))
        }))
    }

    fn read(&self, path: &str) -> anyhow::Result<Bytes> {
        // Compute the hash of this file path
        let mut hasher = HASHER.build_hasher();
        hasher.write(path.to_lowercase().as_bytes());
        let hash = hasher.finish();

        // Look up the file info for this file
        let index = self
            .lut
            .get(&hash)
            .with_context(|| format!("Path not found in index: {}", path))?;
        let file = &self.index.files[*index];

        // Load the bundle
        let bundle_path = format!(
            "/Bundles2/{}.bundle.bin",
            self.index.bundles[file.bundle_index as usize].name
        );
        let bundle_contents = self
            .ggpk
            .read(&bundle_path)
            .with_context(|| format!("Failed to load bundle file: {:?}", bundle_path))?;
        let bundle = BundleParser
            .parse(&bundle_contents)
            .as_anyhow()
            .context("Failed to parse bundle")?;

        // Pull out the file's contents
        let content = bundle.read_range(file.offset as usize, file.size as usize)?;
        Ok(content)
    }
}
