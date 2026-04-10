use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    hash::{BuildHasher, Hasher},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use bytes::Bytes;
use iterators_extended::bucket::Bucket;

use super::FileSystem;
use crate::{
    file_parsers::{
        FileParser,
        bundle::{BundleParser, types::BundleFile},
        bundle_index::{BundleIndexParser, types::BundleIndexFile},
    },
    hasher::murmur64a::BuildMurmurHash64A,
    path::parse_paths,
};

pub struct SteamFS {
    index: BundleIndexFile,
    lut: HashMap<u64, usize>,
    steam_folder: PathBuf,
}

impl SteamFS {
    /// Initialise a file system over a steam folder
    pub fn new(steam_folder: PathBuf) -> Result<Self> {
        let index_path = steam_folder.as_path().join("Bundles2/_.index.bin");
        let index = load_index_file(&index_path).context("Failed to load bundle index")?;

        let lut = index
            .files
            .iter()
            .enumerate()
            .map(|(i, f)| (f.hash, i))
            .collect();

        Ok(Self {
            index,
            lut,
            steam_folder: steam_folder.clone(),
        })
    }
}

impl FileSystem for SteamFS {
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
        let hash_builder = BuildMurmurHash64A { seed: 0x1337b33f };
        let (fileinfos, errors) = paths
            .iter()
            .map(|path| {
                let path = path.as_ref();
                // Compute hash
                let mut hasher = hash_builder.build_hasher();
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
            let bundle_path = self.steam_folder.join(format!(
                "Bundles2/{}.bundle.bin",
                self.index.bundles[bundle_index as usize].name
            ));
            let bundle = load_bundle_content(&bundle_path)
                .with_context(|| format!("Failed to load bundle file: {:?}", bundle_path));

            // Read the file contents
            let contents: Box<dyn Iterator<Item = _>> = match bundle {
                Ok(b) => Box::new(files.into_iter().map(move |(path, file)| {
                    b.read_range(file.offset as usize, file.size as usize)
                        .map(|b| (path, b))
                        .map_err(|e| (path, e))
                })),
                Err(e) => Box::new(
                    files
                        .into_iter()
                        .map(move |(path, _)| Err((path, anyhow!("{:?}", e)))),
                ),
            };

            contents
        });

        // Add on previous errors
        Box::new(errors.into_iter().map(Err).chain(file_contents).map(|r| {
            r.map(|(s, b)| (Cow::Borrowed(s), b))
                .map_err(|(s, e)| (Cow::Borrowed(s), e))
        }))
    }

    fn read(&self, path: &str) -> Result<Bytes> {
        // Compute the hash of this file path
        let hash_builder = BuildMurmurHash64A { seed: 0x1337b33f };
        let mut hasher = hash_builder.build_hasher();
        hasher.write(path.to_lowercase().as_bytes());
        let hash = hasher.finish();

        // Look up the file info for this file
        let file_index = self
            .lut
            .get(&hash)
            .with_context(|| format!("Path not found in index: {}", path))?;
        let file = &self.index.files[*file_index];

        // Load the bundle
        let bundle_path = self.steam_folder.join(format!(
            "Bundles2/{}.bundle.bin",
            self.index.bundles[file.bundle_index as usize].name
        ));
        let bundle = load_bundle_content(&bundle_path)
            .with_context(|| format!("Failed to load bundle file: {:?}", bundle_path))?;

        // Pull out the file's contents
        let content = bundle
            .read_range(file.offset as usize, file.size as usize)
            .context("Failed to read bytes from bundle")?;
        Ok(content)
    }
}

/// Load an index file from disk
fn load_index_file(path: &Path) -> Result<BundleIndexFile> {
    let index_content = load_bundle_content(path)
        .context("Failed to read bundle index")?
        .read_all()
        .context("Failed to read bytes from bundle")?;

    BundleIndexParser
        .parse(&index_content)
        .as_anyhow()
        .context("Failed to parse bundle as index")
}

/// Load a bundle file from disk
fn load_bundle_content(path: &Path) -> Result<BundleFile> {
    let bundle_content = fs::read(path).context("Failed to read bundle file")?;

    let bundle = BundleParser
        .parse(&bundle_content)
        .as_anyhow()
        .context("Failed to parse bundle")?;

    Ok(bundle)
}

/// Helper to find steam installs in common locations
pub fn steam_folder_search(patch: &str) -> Option<PathBuf> {
    let home = dirs::home_dir().unwrap();
    let game = match patch {
        "1" => "Path of Exile",
        "2" => "Path of Exile 2",
        _ => return None,
    };
    [
        home.join(".local/share/Steam/steamapps/common"),
        home.join("Library/Application Support/Steam/steamapps/common"),
        PathBuf::from("C:\\Program Files (x86)\\Grinding Gear Games"),
        PathBuf::from("/mnt/e/SteamLibrary/steamapps/common"),
    ]
    .iter()
    .map(|p| p.join(game))
    .find(|p| p.exists())
}
