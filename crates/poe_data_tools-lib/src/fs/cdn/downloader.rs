use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    time::Duration,
};

use bytes::Bytes;
use iterators_extended::bucket::Bucket;
use reqwest::blocking::Client;
use url::Url;
use winnow::{
    Parser,
    binary::{le_u8, length_repeat, length_take},
    combinator::terminated,
    token::take,
};

use crate::{
    file_parsers::{error::ParseError, shared::winnow::WinnowParser},
    fs::{Result, error::Error as FSError},
};

#[derive(Debug, thiserror::Error, Clone)]
pub enum CDNError {
    #[error("response has no etag header")]
    NoEtag,
    #[error("response contains invalid etag header")]
    BadEtag,
    #[error("CDN provided no valid URLs")]
    NoUrls,
}

/// A cache folder for a specific game version
#[derive(Debug)]
struct CacheFolder {
    path: PathBuf,
    /// patch version components [major, minor, patch, ...]
    patch_parts: Vec<u64>,
}

impl PartialEq for CacheFolder {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for CacheFolder {}

impl PartialOrd for CacheFolder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CacheFolder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.patch_parts.cmp(&other.patch_parts)
    }
}

impl CacheFolder {
    fn from_path(path: PathBuf) -> Result<Self> {
        let folder_name = path.file_name().and_then(|f| f.to_str()).ok_or_else(|| {
            FSError::InvalidConfig(format!("Cache path has no/non-utf8 folder: {path:?}"))
        })?;

        let patch_parts = parse_patch_parts(folder_name).map_err(|_| {
            FSError::InvalidConfig(format!(
                "cache folder could not be parsed as patch version: {folder_name:?}"
            ))
        })?;

        Ok(Self { path, patch_parts })
    }

    /// Get the cache path for a file
    fn get_path(&self, path_stub: &Path) -> (PathBuf, PathBuf) {
        let file_path = self.path.join(path_stub);
        let etag_path = file_path.with_added_extension("etag");

        (file_path, etag_path)
    }

    /// Search for the file in this cache
    fn search(&self, path_stub: &Path) -> Option<(PathBuf, PathBuf)> {
        let (file_path, etag_path) = self.get_path(path_stub);

        (file_path.exists() && etag_path.exists()).then_some((file_path, etag_path))
    }
}

/// "1.2.3.4" -> [1, 2, 3, 4]
fn parse_patch_parts(filename: &str) -> Result<Vec<u64>, ParseError> {
    filename
        .split('.')
        .map(|x| x.parse::<u64>().map_err(ParseError::other))
        .collect()
}

struct CacheConfig {
    /// Folder containing all patch-specific folders
    _root: PathBuf,
    /// Cache for the targetted game version
    primary: CacheFolder,
    /// Folders to search when primary cache misses
    fallbacks_old: Vec<CacheFolder>,
    fallbacks_new: Vec<CacheFolder>,
}

impl CacheConfig {
    fn from_primary_cache_path(path: PathBuf) -> Result<Self> {
        let primary = CacheFolder::from_path(path.clone())?;

        let root = path.parent().ok_or_else(|| {
            FSError::InvalidConfig(format!("cache folder has no parent: {path:?}"))
        })?;

        let (fallbacks_old, fallbacks_new) = get_fallback_cache_dirs(root, &primary);

        Ok(Self {
            _root: root.to_owned(),
            primary,
            fallbacks_old,
            fallbacks_new,
        })
    }

    /// Search for the given path stub in fallback cache folders
    /// Returns all matching file & etag paths
    fn search_fallbacks(&self, path_stub: &Path) -> impl Iterator<Item = (PathBuf, PathBuf)> {
        self.fallback_caches().flat_map(|c| c.search(path_stub))
    }

    /// Iterate over all cache folders in order: past (new->old) then future (old -> new)
    // TODO: Interleave fallbacks in "distance from current patch" order
    fn fallback_caches(&self) -> impl Iterator<Item = &CacheFolder> {
        self.fallbacks_old.iter().chain(&self.fallbacks_new)
    }
}

/// Prepares two lists of backup cache dirs, both ordered by recency to the current patch
fn get_fallback_cache_dirs(
    root_dir: &Path,
    primary_cache: &CacheFolder,
) -> (Vec<CacheFolder>, Vec<CacheFolder>) {
    // Read adjacent paths, skip any that fail
    let sibling_paths = root_dir
        .read_dir()
        .into_iter()
        .flat_map(|r| r.into_iter())
        .flatten();

    // Split cache dirs into older/newer patches
    let [mut old, mut new] = sibling_paths
        .filter_map(|p| CacheFolder::from_path(p.path()).ok())
        // Don't include primary in fallbacks
        .filter(|cache| cache != primary_cache)
        .bucket_arr(|cache| if cache < primary_cache { 0 } else { 1 });

    // Sort in chronoligical order away from current path dir
    old.sort();
    old.reverse();
    new.sort();

    (old, new)
}

/// Bundle loader backed by GGG's CDN + local cache
pub struct CDNLoader {
    /// CDN Url
    base_url: Url,
    /// Place where any new files will be downloaded to
    cache: CacheConfig,
}

impl CDNLoader {
    pub fn new(base_url: &Url, cache_dir: &str) -> Result<Self> {
        // <cache_path>/<cdn_url>/<patch>
        let cache_dir = PathBuf::from(cache_dir).join(format!(
            "{}{}",
            base_url
                .domain()
                .ok_or_else(|| FSError::InvalidConfig(format!("Invalid CDN URL: {base_url:?}")))?,
            base_url.path()
        ));

        let cache = CacheConfig::from_primary_cache_path(cache_dir)?;

        Ok(Self {
            base_url: base_url.clone(),
            cache,
        })
    }

    /// Get the CDN url for a file
    fn build_url(&self, path_stub: &Path) -> Result<Url> {
        self.base_url
            .join(path_stub.to_str().ok_or_else(|| {
                FSError::InvalidConfig(format!("path stub contains non-UTF8 chars: {path_stub:?}"))
            })?)
            .map_err(|e| FSError::InvalidConfig(format!("invalid URL: {e:?}")))
    }

    /// Loads the contents of the bundle file. Either reads from the local cache or from the CDN if
    /// it's not cached.
    pub fn load(&self, path_stub: &Path) -> Result<Bytes> {
        let url = self.build_url(path_stub)?;

        // If already cached, assume nothing has changed due to version immutability
        if let Some((cache_path, _)) = self.cache.primary.search(path_stub) {
            log::debug!("Using cached bundle: {:?}", cache_path);
            let bytes = fs::read(&cache_path)?;
            return Ok(Bytes::from(bytes));
        }

        // Short timeout for initial connection, but none for transfer to allow for fetching large
        // files on a poor network connection
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(None)
            .build()?;

        // Ask server for the etag so we can check against cache
        let etag = {
            let resp = client
                .head(url.clone())
                // TODO: Maybe be a little friendlier if we can do this? But would be more complex
                // code
                // .header("If-None-Match", fallback_etag.clone())
                .send()?
                .error_for_status()?;

            // TODO: (above)
            // Also with this
            // let is_match = resp.status() == StatusCode::NOT_MODIFIED
            //     || (resp.status() == StatusCode::OK && resp_etag == fallback_etag);

            resp.headers()
                .get("etag")
                .ok_or(CDNError::NoEtag)?
                .to_str()
                .map_err(|_| CDNError::BadEtag)?
                .to_owned()
        };

        let fallback = self
            .cache
            .search_fallbacks(path_stub)
            .find(|(_, etag_path)| {
                fs::read_to_string(etag_path).is_ok_and(|fallback_etag| fallback_etag == etag)
            });

        let (cache_file_path, cache_etag_path) = self.cache.primary.get_path(path_stub);

        let bytes = if let Some((fallback_file_path, fallback_etag_path)) = fallback {
            // Our cached version is the same, copy it to the current version's directory
            log::debug!(
                "Using cached bundle from different patch: {:?}",
                fallback_file_path
            );
            fs::create_dir_all(
                cache_file_path
                    .parent()
                    .expect("path inside cache always has a parent folder"),
            )?;

            fs::copy(&fallback_file_path, &cache_file_path)?;
            fs::copy(fallback_etag_path, cache_etag_path)?;

            Bytes::from(fs::read(&cache_file_path)?)
        } else {
            // No candidate cached version, download from CDN
            log::info!("Downloading bundle: {}", url);
            let resp = client.get(url).send()?;

            let etag = resp
                .headers()
                .get("etag")
                .ok_or(CDNError::NoEtag)?
                .to_str()
                .map_err(|_| CDNError::BadEtag)?
                .to_owned();

            let bytes = resp.bytes()?;

            fs::create_dir_all(
                cache_file_path
                    .parent()
                    .expect("path inside cache always has a parent folder"),
            )?;

            fs::write(&cache_file_path, &bytes)?;
            fs::write(cache_etag_path, etag.as_bytes())?;

            bytes
        };

        Ok(bytes)
    }

    /// Async version of load
    // TODO: Mutex to prevent multiple concurrent downloads of same file. Shouldn't happen under
    // normal circumstances
    pub async fn load_async(&self, path_stub: &Path) -> Result<Bytes> {
        let url = self.build_url(path_stub)?;

        // Short timeout for initial connection, but none for transfer to allow for fetching large
        // files on a poor network connection
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .build()?;

        // If already cached, assume nothing has changed due to version immutability
        if let Some((cache_path, _)) = self.cache.primary.search(path_stub) {
            log::debug!("Using cached bundle: {:?}", cache_path);
            let bytes = tokio::fs::read(&cache_path).await?;
            return Ok(Bytes::from(bytes));
        }

        // Ask server for the etag so we can check against cache
        let etag = {
            let resp = client
                .head(url.clone())
                // TODO: Maybe be a little friendlier if we can do this? But would be more complex
                // code
                // .header("If-None-Match", fallback_etag.clone())
                .send()
                .await?
                .error_for_status()?;

            // TODO: (above)
            // Also with this
            // let is_match = resp.status() == StatusCode::NOT_MODIFIED
            //     || (resp.status() == StatusCode::OK && resp_etag == fallback_etag);

            resp.headers()
                .get("etag")
                .ok_or(CDNError::NoEtag)?
                .to_str()
                .map_err(|_| CDNError::BadEtag)?
                .to_owned()
        };

        let fallback = self
            .cache
            .search_fallbacks(path_stub)
            .find(|(_, etag_path)| {
                fs::read_to_string(etag_path).is_ok_and(|fallback_etag| fallback_etag == etag)
            });

        let (cache_file_path, cache_etag_path) = self.cache.primary.get_path(path_stub);

        let bytes = if let Some((fallback_file_path, fallback_etag_path)) = fallback {
            // Our cached version is the same, copy it to the current version's directory
            log::debug!(
                "Using cached bundle from different patch: {:?}",
                fallback_file_path
            );
            tokio::fs::create_dir_all(
                cache_file_path
                    .parent()
                    .expect("path inside cache always has a parent folder"),
            )
            .await?;

            tokio::fs::copy(&fallback_file_path, &cache_file_path).await?;
            tokio::fs::copy(fallback_etag_path, cache_etag_path).await?;

            Bytes::from(fs::read(&cache_file_path)?)
        } else {
            // No candidate cached version, download from CDN
            log::info!("Downloading bundle: {}", url);
            let resp = client.get(url).send().await?;

            let etag = resp
                .headers()
                .get("etag")
                .ok_or(CDNError::NoEtag)?
                .to_str()
                .map_err(|_| CDNError::BadEtag)?
                .to_owned();

            let bytes = resp.bytes().await?;

            tokio::fs::create_dir_all(
                cache_file_path
                    .parent()
                    .expect("path inside cache always has a parent folder"),
            )
            .await?;

            tokio::fs::write(&cache_file_path, &bytes).await?;
            tokio::fs::write(cache_etag_path, etag.as_bytes()).await?;

            bytes
        };

        Ok(bytes)
    }
}

/// Get the base URL for the CDN for the provided game version. Uses cached version if available.
pub fn cdn_base_url(cache_dir: &Path, version: &str) -> Result<Url> {
    // Check cache for version URL
    let cache_dir = cache_dir.join("cdn_url");
    let cache_file = cache_dir.join(version);

    // If we have a recently cached version, just use that instead
    if cache_file.exists()
        && fs::metadata(&cache_file)?
            .modified()?
            .elapsed()
            .map(|d| d.as_secs() < 3600)
            .unwrap_or(false)
    {
        let url =
            Url::parse(fs::read_to_string(&cache_file)?.as_str()).map_err(ParseError::other)?;
        log::debug!("Using cached CDN URL: {}", url);
        return Ok(url);
    }

    let url = match version {
        // Latest PoE 1
        "1" => cur_url("patch.pathofexile.com:12995".to_string(), &[1, 6])?,
        // Latest PoE 2
        "2" => cur_url("patch.pathofexile2.com:13060".to_string(), &[1, 7])?,
        // Specific PoE 1 patch
        v if v.starts_with("3.") => Url::parse(format!("https://patch.poecdn.com/{}/", v).as_str())
            .map_err(ParseError::other)?,
        // Specific PoE 2 patch
        v if v.starts_with("4.") => {
            Url::parse(format!("https://patch-poe2.poecdn.com/{}/", v).as_str())
                .map_err(ParseError::other)?
        }
        // Invalid patch
        _ => panic!("Invalid version provided"),
    };

    fs::create_dir_all(&cache_dir)?;
    fs::write(&cache_file, url.as_str())?;
    log::debug!("Refreshed CDN URL: {}", url);
    Ok(url)
}

fn parse_response<'a>() -> impl WinnowParser<&'a [u8], Vec<String>> {
    length_repeat(
        terminated(le_u8, take(33_usize)), //
        parse_utf16_string(),
    )
}

fn parse_utf16_string<'a>() -> impl WinnowParser<&'a [u8], String> {
    winnow::trace!(
        "parse_utf16_string",
        length_take(le_u8.map(|l| l * 2)).try_map(String::from_utf16le)
    )
}

/// Fetch the current latest version of the game
fn cur_url(host: String, send: &[u8]) -> Result<Url> {
    // Fetch data from the CDN
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(send)?;
    let mut buf = [0; 1024];
    let read = stream.read(&mut buf)?;

    let strings = parse_response()
        .parse_next(&mut &buf[..read])
        .map_err(|_| CDNError::NoUrls)?;

    // Grab the first one and return it as a URL
    let url = strings
        .into_iter()
        .flat_map(|s| Url::parse(&s).ok())
        .next()
        .ok_or(CDNError::NoUrls)?;

    Ok(url)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::TempDir;

    use super::*;

    fn write_file(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn setup_fake_cache() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let cache1 = root.join("1.1.1.1");
        let cache2 = root.join("2.2.2.2");
        let stub1 = Path::new("hello/world.txt");
        let stub2 = Path::new("bing.bong");

        write_file(&cache1.join(stub1), "");
        write_file(
            &cache1.join(stub1).with_added_extension("etag"),
            "hello world",
        );

        write_file(&cache2.join(stub2), "");
        write_file(
            &cache2.join(stub2).with_added_extension("etag"),
            "hello world",
        );

        temp
    }

    #[test]
    fn test_cache_folder() {
        let temp = setup_fake_cache();

        let cache_path = temp.path().join("1.1.1.1");
        let cache = CacheFolder::from_path(cache_path).unwrap();

        assert_eq!(cache.patch_parts, [1, 1, 1, 1]);

        assert!(cache.search(Path::new("hello/world.txt")).is_some());
        assert!(cache.search(Path::new("bing.bong")).is_none());

        drop(temp);
    }

    #[test]
    fn test_cache() {
        let temp = setup_fake_cache();

        let primary_cache_path = temp.path().join("1.1.1.1");
        let cache = CacheConfig::from_primary_cache_path(primary_cache_path).unwrap();

        assert_eq!(cache.fallback_caches().count(), 1);

        assert_eq!(
            cache.search_fallbacks(Path::new("hello/world.txt")).count(),
            0
        );
        assert_eq!(cache.search_fallbacks(Path::new("bing.bong")).count(), 1);
    }
}
