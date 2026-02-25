use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, bail};
use bytes::Bytes;
use reqwest::blocking::Client;
use url::Url;
use winnow::{
    Parser,
    binary::{le_u8, length_repeat, length_take},
    combinator::terminated,
    token::take,
};

use crate::file_parsers::shared::winnow::{TraceHelper, WinnowParser};

pub struct CDNLoader {
    base_url: Url,
    cache_dir: String,
}

impl CDNLoader {
    pub fn new(base_url: &Url, cache_dir: &str) -> Self {
        Self {
            base_url: base_url.clone(),
            cache_dir: cache_dir.to_string(),
        }
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
        let cache_path =
            PathBuf::from(&self.cache_dir).join(url.to_string().trim_start_matches("https://"));
        if let Ok(bytes) = fs::read(&cache_path) {
            //eprintln!("Loading bundle from cache: {:?}", path_stub);
            return Ok(Bytes::from(bytes));
        }

        eprintln!("Downloading bundle: {}", url);
        // Short timeout for initial connection, but none for transfer to allow for fetching large
        // files on a poor network connection
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(None)
            .build()?;
        let bytes = client.get(url).send()?.error_for_status()?.bytes()?;
        // Save data to file - data first then ETag in case of failure mid-download
        fs::create_dir_all(cache_path.parent().context("Failed to get path parent")?)?;
        fs::write(&cache_path, &bytes)?;

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
        eprintln!("Using cached CDN URL: {}", url);
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
    eprintln!("Refreshed CDN URL: {}", url);
    Ok(url)
}

fn parse_response<'a>() -> impl WinnowParser<&'a [u8], Vec<String>> {
    length_repeat(
        terminated(le_u8, take(33_usize)), //
        parse_utf16_string(),
    )
}

fn parse_utf16_string<'a>() -> impl WinnowParser<&'a [u8], String> {
    length_take(le_u8.map(|l| l * 2))
        .try_map(String::from_utf16le)
        .trace("parse_utf16_string")
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
