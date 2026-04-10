use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, bail};
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

use crate::file_parsers::shared::winnow::WinnowParser;

pub struct CDNLoader {
    /// CDN Url
    base_url: Url,
    /// Place where any new files will be downloaded to
    cache_dir: PathBuf,
    /// Caches for previous game versions to search before downloading new files
    fallback_cache_dirs_old: Vec<PathBuf>,
    fallback_cache_dirs_new: Vec<PathBuf>,
}

/// "1.2.3.4" -> [1, 2, 3, 4]
fn parse_patch_parts(filename: &str) -> anyhow::Result<Vec<u64>> {
    filename
        .split('.')
        .map(|x| {
            x.parse::<u64>()
                .with_context(|| format!("Failed to parse filename part as u64: {x:?}"))
        })
        .collect()
}

/// Prepares two lists of backup cache dirs, both ordered by recency to the current patch
fn get_fallback_cache_dirs(cache_dir: &Path) -> anyhow::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let current_patch = parse_patch_parts(cache_dir.file_name().unwrap().to_str().unwrap())?;
    let parent = cache_dir.parent().unwrap();

    // Split cache dirs into older/newer patches
    let [mut old, mut new] = parent
        .read_dir()
        .context("Failed to read cache dir")?
        .filter_map(Result::ok)
        .filter_map(|p| {
            let binding = p.file_name();
            let filename = binding.to_str().unwrap();
            parse_patch_parts(filename)
                .ok()
                .map(|patch| (patch, p.path()))
        })
        .filter(|(patch, _)| *patch != current_patch)
        .bucket_arr(|(patch, _)| if *patch < current_patch { 0 } else { 1 });

    old.sort_unstable_by_key(|(patch, _)| patch.clone());
    old.reverse();
    let old = old.into_iter().map(|(_, path)| path).collect();

    new.sort_unstable_by_key(|(patch, _)| patch.clone());
    let new = new.into_iter().map(|(_, path)| path).collect();

    Ok((old, new))
}

impl CDNLoader {
    pub fn new(base_url: &Url, cache_dir: &str) -> anyhow::Result<Self> {
        // <cache_path>/<cdn_url>/<patch>
        let cache_dir = PathBuf::from(cache_dir).join(format!(
            "{}{}",
            base_url.domain().context("CDN URL has no domain")?,
            base_url.path()
        ));

        let (fallback_cache_dirs_old, fallback_cache_dirs_new) =
            get_fallback_cache_dirs(&cache_dir)?;

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

    /// Loads the contents of the bundle file. Either reads from the local cache or from the CDN if
    /// it's not cached.
    pub fn load(&self, path_stub: &Path) -> anyhow::Result<Bytes> {
        let url = self.base_url.join(
            path_stub
                .to_str()
                .context("Failed to parse path as string")?,
        )?;

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
                .try_find(|(_, fallback_etag)| -> anyhow::Result<_> {
                    let resp = client
                        .head(url.clone())
                        .header("If-None-Match", fallback_etag.clone())
                        .send()?
                        .error_for_status()?;
                    let resp_etag = resp.headers().get("etag").context("no etag")?.to_str()?;

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
            fs::create_dir_all(cache_path.parent().context("Failed to get path parent")?)?;
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
                .context("no etag")?
                .to_str()?
                .to_owned();

            let bytes = resp.bytes()?;
            fs::create_dir_all(cache_path.parent().context("Failed to get path parent")?)?;
            fs::write(&cache_path, &bytes)?;
            fs::write(cache_path.with_added_extension("etag"), etag.as_bytes())?;

            bytes
        };

        Ok(bytes)
    }

    /// Async version of load
    // TODO: Mutex to prevent multiple concurrent downloads of same file. Shouldn't happen under
    // normal circumstances
    pub async fn load_async(&self, path_stub: &Path) -> anyhow::Result<Bytes> {
        let url = self.base_url.join(
            path_stub
                .to_str()
                .context("Failed to parse path as string")?,
        )?;

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
            let resp_etag = resp.headers().get("etag").context("no etag")?.to_str()?;

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
            tokio::fs::create_dir_all(cache_path.parent().context("Failed to get path parent")?)
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
                .context("no etag")?
                .to_str()?
                .to_owned();

            let bytes = resp.bytes().await?;
            tokio::fs::create_dir_all(cache_path.parent().context("Failed to get path parent")?)
                .await?;
            tokio::fs::write(&cache_path, &bytes).await?;
            tokio::fs::write(cache_path.with_added_extension("etag"), etag.as_bytes()).await?;

            bytes
        };

        Ok(bytes)
    }
}

pub fn cdn_base_url(cache_dir: &Path, version: &str) -> anyhow::Result<Url> {
    // Check cache for version URL
    let cache_dir = cache_dir.join("cdn_url");
    let cache_file = cache_dir.join(version);

    // If we have a recently cached version, just use that instead
    if cache_file.exists() && fs::metadata(&cache_file)?.modified()?.elapsed()?.as_secs() < 3600 {
        let url = Url::parse(fs::read_to_string(&cache_file)?.as_str())
            .with_context(|| "Failed to parse URL")?;
        log::debug!("Using cached CDN URL: {}", url);
        return Ok(url);
    }

    let url = match version {
        // Latest PoE 1
        "1" => cur_url("patch.pathofexile.com:12995".to_string(), &[1, 6]),
        // Latest PoE 2
        "2" => cur_url("patch.pathofexile2.com:13060".to_string(), &[1, 7]),
        // Specific PoE 1 patch
        v if v.starts_with("3.") => Url::parse(format!("https://patch.poecdn.com/{}/", v).as_str())
            .with_context(|| "Failed to parse URL"),
        // Specific PoE 2 patch
        v if v.starts_with("4.") => {
            Url::parse(format!("https://patch-poe2.poecdn.com/{}/", v).as_str())
                .with_context(|| "Failed to parse URL")
        }
        // Invalid patch
        _ => panic!("Invalid version provided"),
    }
    .with_context(|| format!("Failed to get URL for version: {}", version))?;

    fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;
    fs::write(&cache_file, url.as_str()).context("Failed to write URL to cache")?;
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
fn cur_url(host: String, send: &[u8]) -> anyhow::Result<Url> {
    // Fetch data from the CDN - todo: looks like this returns a list of URLs. Might need to use a
    // streaming-style parsing instead of just reading 1Kb down the line if there's a bunch of
    // strings
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(send)?;
    let mut buf = [0; 1024];
    let read = stream.read(&mut buf)?;

    // Parse the response
    let strings = if let Ok(strings) = parse_response().parse_next(&mut &buf[..read]) {
        strings
    } else {
        bail!("Failed to parse URLs from CDN")
    };

    // Grab the first one and return it as a URL
    strings
        .into_iter()
        .map(|s| Url::parse(&s).expect("Failed to parse URL"))
        .next()
        .with_context(|| "No URLs returned from CDN")
}
