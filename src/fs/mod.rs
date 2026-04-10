pub mod cdn;
pub mod ggpk;
pub mod steam;

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use bytes::Bytes;
use cdn::CDNFS;
use enum_dispatch::enum_dispatch;
use steam::SteamFS;
use url::Url;

use crate::fs::ggpk::GGPKBundleFS;

#[enum_dispatch]
pub trait FileSystem {
    /// Lists all paths in the index
    fn list(&self) -> Box<dyn Iterator<Item = String> + '_>;

    /// Read many files at once, optimising batch loads. Does not preserve order of paths given.
    #[allow(clippy::type_complexity)]
    fn batch_read<'a>(
        &'a self,
        paths: &'a [impl AsRef<str>],
    ) -> Box<dyn Iterator<Item = (Cow<'a, str>, anyhow::Result<Bytes>)> + 'a>;

    /// Read a single file's contents
    fn read(&self, path: &str) -> anyhow::Result<Bytes>;
}

#[enum_dispatch(FileSystem)]
pub enum FS {
    Steam(SteamFS),
    CDN(CDNFS),
    GGPK(GGPKBundleFS),
}

impl FS {
    /// Initialise a file system over a steam folder
    pub fn from_steam(steam_folder: PathBuf) -> anyhow::Result<Self> {
        SteamFS::new(steam_folder).map(Self::Steam)
    }

    /// Initialise a file system using the CDN backend
    pub fn from_cdn(base_url: &Url, cache_dir: &Path) -> anyhow::Result<FS> {
        CDNFS::new(base_url, cache_dir).map(Self::CDN)
    }

    /// Initialise a file system over a standalone GGPK file
    pub fn from_ggpk(ggpk_path: &Path) -> anyhow::Result<FS> {
        GGPKBundleFS::new(ggpk_path).map(Self::GGPK)
    }
}
