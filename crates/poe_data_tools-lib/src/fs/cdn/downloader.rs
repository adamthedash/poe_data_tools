use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use bytes::Bytes;
use iterators_extended::bucket::Bucket;
use reqwest::{StatusCode, blocking::Client};
use url::Url;
use winnow::{
    Parser,
    binary::{le_u8, length_repeat, length_take},
    combinator::terminated,
    token::take,
};

use crate::{
    file_parsers::shared::winnow::WinnowParser,
    fs::{Result, error::Error as FSError},
};

/// Bundle loader backed by GGG's CDN + local cache
pub struct CDNLoader {
    /// CDN Url
    base_url: Url,
    /// Place where any new files will be downloaded to
    // TODO: Pre-process config stuff once into struct so we don't need to keep checking on use?
    cache_dir: PathBuf,
    /// Caches for previous game versions to search before downloading new files
    fallback_cache_dirs_old: Vec<PathBuf>,
    fallback_cache_dirs_new: Vec<PathBuf>,
}

/// "1.2.3.4" -> [1, 2, 3, 4]
fn parse_patch_parts(filename: &str) -> Result<Vec<u64>> {
    filename
        .split('.')
        .map(|x| {
            x.parse::<u64>()
                .with_context(|| format!("Failed to parse filename part as u64: {x:?}"))
                .map_err(|e| FSError::Parse(Arc::new(e)))
        })
        .collect()
}

/// Prepares two lists of backup cache dirs, both ordered by recency to the current patch
fn get_fallback_cache_dirs(cache_dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let Some(cache_folder_name) = cache_dir.file_name().and_then(|f| f.to_str()) else {
        log::warn!("invalid cache path: {cache_dir:?}");
        return (vec![], vec![]);
    };

    let Ok(current_patch) = parse_patch_parts(cache_folder_name) else {
        log::warn!("invalid cache folder name: {cache_dir:?}");
        return (vec![], vec![]);
    };

    let Some(parent) = cache_dir.parent() else {
        log::warn!("cache has no parent folder: {cache_dir:?}");
        return (vec![], vec![]);
    };

    // Read adjacent paths, skip any that fail
    let sibling_paths = parent
        .read_dir()
        .into_iter()
        .flat_map(|r| r.into_iter())
        .flatten();

    // Split cache dirs into older/newer patches
    let [mut old, mut new] = sibling_paths
        .filter_map(|p| {
            let binding = p.file_name();
            let filename = binding.to_str().unwrap();
            parse_patch_parts(filename)
                .ok()
                .map(|patch| (patch, p.path()))
        })
        .filter(|(patch, _)| *patch != current_patch)
        .bucket_arr(|(patch, _)| if *patch < current_patch { 0 } else { 1 });

    // Sort in chronoligical order away from current path dir
    old.sort_unstable_by_key(|(patch, _)| patch.clone());
    old.reverse();
    let old = old.into_iter().map(|(_, path)| path).collect();

    new.sort_unstable_by_key(|(patch, _)| patch.clone());
    let new = new.into_iter().map(|(_, path)| path).collect();

    (old, new)
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

        let (fallback_cache_dirs_old, fallback_cache_dirs_new) =
            get_fallback_cache_dirs(&cache_dir);

        Ok(Self {
            base_url: base_url.clone(),
            cache_dir,
            fallback_cache_dirs_old,
            fallback_cache_dirs_new,
        })
    }

    /// Finds cached files from previous versions
    fn get_fallbacks(&self, path_stub: &Path) -> impl Iterator<Item = (PathBuf, String)> {
        [&self.fallback_cache_dirs_old, &self.fallback_cache_dirs_new]
            .into_iter()
            .flat_map(move |cache_dirs| {
                cache_dirs
                    .iter()
                    .flat_map(|cache_dir| {
                        let file_path = cache_dir.join(path_stub);
                        let etag_path = file_path.with_added_extension("etag");

                        fs::read_to_string(&etag_path)
                            .ok()
                            .map(|etag| (file_path, etag))
                    })
                    .next()
            })
    }

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
        let cache_path = self.cache_dir.join(path_stub);
        if let Ok(bytes) = fs::read(&cache_path) {
            log::debug!("Using cached bundle: {:?}", cache_path);
            return Ok(Bytes::from(bytes));
        }

        // Short timeout for initial connection, but none for transfer to allow for fetching large
        // files on a poor network connection
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(None)
            .build()?;

        // Look at fallback caches and see if there's any that match the CDN's
        let fallback =
            self.get_fallbacks(path_stub)
                .try_find(|(_, fallback_etag)| -> Result<_, FSError> {
                    let resp = client
                        .head(url.clone())
                        .header("If-None-Match", fallback_etag.clone())
                        .send()?
                        .error_for_status()?;

                    let resp_etag = resp
                        .headers()
                        .get("etag")
                        .ok_or_else(|| {
                            FSError::InvalidResponse("response has no etag header".to_owned())
                        })?
                        .to_str()
                        .map_err(|e| {
                            FSError::InvalidResponse(format!(
                                "response contains invalid etag: {e:?}"
                            ))
                        })?;

                    let is_match = resp.status() == StatusCode::NOT_MODIFIED
                        || (resp.status() == StatusCode::OK && resp_etag == fallback_etag);

                    Ok(is_match)
                })?;

        let bytes = if let Some((fallback_path, _)) = fallback {
            // Our cached version is the same, copy it to the current version's directory
            log::debug!(
                "Using cached bundle from different patch: {:?}",
                fallback_path
            );
            fs::create_dir_all(cache_path.parent().ok_or_else(|| {
                FSError::InvalidConfig(format!("cache has no parent folder: {cache_path:?}"))
            })?)?;

            fs::copy(&fallback_path, &cache_path)?;
            fs::copy(
                fallback_path.with_added_extension("etag"),
                cache_path.with_added_extension("etag"),
            )?;

            Bytes::from(fs::read(&cache_path)?)
        } else {
            // No candidate cached version, download from CDN
            log::info!("Downloading bundle: {}", url);
            let resp = client.get(url).send()?;

            let etag = resp
                .headers()
                .get("etag")
                .ok_or_else(|| FSError::InvalidResponse("response has no etag header".to_owned()))?
                .to_str()
                .map_err(|e| {
                    FSError::InvalidResponse(format!("response contains invalid etag: {e:?}"))
                })?
                .to_owned();

            let bytes = resp.bytes()?;
            fs::create_dir_all(cache_path.parent().ok_or_else(|| {
                FSError::InvalidConfig(format!("cache has no parent folder: {cache_path:?}"))
            })?)?;
            fs::write(&cache_path, &bytes)?;
            fs::write(cache_path.with_added_extension("etag"), etag.as_bytes())?;

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
        let cache_path = self.cache_dir.join(path_stub);
        if let Ok(bytes) = tokio::fs::read(&cache_path).await {
            log::debug!("Using cached bundle: {:?}", cache_path);
            return Ok(Bytes::from(bytes));
        }

        // Look at fallback caches and see if there's any that match the CDN's
        let mut fallbacks = self.get_fallbacks(path_stub);
        let fallback = loop {
            let Some((fallback_path, fallback_etag)) = fallbacks.next() else {
                break None;
            };

            let resp = client
                .head(url.clone())
                .header("If-None-Match", fallback_etag.clone())
                .send()
                .await?
                .error_for_status()?;

            let resp_etag = resp
                .headers()
                .get("etag")
                .ok_or_else(|| FSError::InvalidResponse("response has no etag header".to_owned()))?
                .to_str()
                .map_err(|e| {
                    FSError::InvalidResponse(format!("response contains invalid etag: {e:?}"))
                })?;

            if resp.status() == StatusCode::NOT_MODIFIED
                || (resp.status() == StatusCode::OK && resp_etag == fallback_etag)
            {
                break Some((fallback_path, fallback_etag));
            }
        };

        let bytes = if let Some((fallback_path, _)) = fallback {
            // Our cached version is the same, copy it to the current version's directory
            log::debug!(
                "Using cached bundle from different patch: {:?}",
                fallback_path
            );
            tokio::fs::create_dir_all(cache_path.parent().ok_or_else(|| {
                FSError::InvalidConfig(format!("cache has no parent folder: {cache_path:?}"))
            })?)
            .await?;
            tokio::fs::copy(&fallback_path, &cache_path).await?;
            tokio::fs::copy(
                fallback_path.with_added_extension("etag"),
                cache_path.with_added_extension("etag"),
            )
            .await?;

            Bytes::from(fs::read(&cache_path)?)
        } else {
            // No candidate cached version, download from CDN
            log::debug!("Downloading bundle: {}", url);
            let resp = client.get(url).send().await?;

            let etag = resp
                .headers()
                .get("etag")
                .ok_or_else(|| FSError::InvalidResponse("response has no etag header".to_owned()))?
                .to_str()
                .map_err(|e| {
                    FSError::InvalidResponse(format!("response contains invalid etag: {e:?}"))
                })?
                .to_owned();

            let bytes = resp.bytes().await?;
            tokio::fs::create_dir_all(cache_path.parent().ok_or_else(|| {
                FSError::InvalidConfig(format!("cache has no parent folder: {cache_path:?}"))
            })?)
            .await?;
            tokio::fs::write(&cache_path, &bytes).await?;
            tokio::fs::write(cache_path.with_added_extension("etag"), etag.as_bytes()).await?;

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
        let url = Url::parse(fs::read_to_string(&cache_file)?.as_str())
            .with_context(|| "Failed to parse URL")
            .map_err(|e| FSError::Parse(Arc::new(e)))?;
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
            .with_context(|| "Failed to parse URL")
            .map_err(|e| FSError::Parse(Arc::new(e)))?,
        // Specific PoE 2 patch
        v if v.starts_with("4.") => {
            Url::parse(format!("https://patch-poe2.poecdn.com/{}/", v).as_str())
                .with_context(|| "Failed to parse URL")
                .map_err(|e| FSError::Parse(Arc::new(e)))?
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

    let Ok(strings) = parse_response().parse_next(&mut &buf[..read]) else {
        return Err(FSError::InvalidResponse(
            "Failed to parse URLs from CDN".to_owned(),
        ));
    };

    // Grab the first one and return it as a URL
    strings
        .into_iter()
        .flat_map(|s| Url::parse(&s).ok())
        .next()
        .ok_or(FSError::InvalidResponse(
            "No valid URLs returned from CDN".to_owned(),
        ))
}
