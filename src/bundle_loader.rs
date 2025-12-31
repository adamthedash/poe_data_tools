use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{bail, Context};
use bytes::Bytes;
use nom::{bytes::complete::take, multi::count, number::complete::le_u8, IResult};
use reqwest::blocking::Client;
use url::Url;

/// Bundle file loader which pulls from the PoE CDN. Files are cached locally for re-use.  
///
/// # Examples
///
/// Load an art file from a specific version of PoE 2
/// ```
/// let base_url = Url::parse("https://patch-poe2.poecdn.com/4.4.0.3.9/").unwrap();
/// let loader = CDNLoader::new(&base_url, ".cache");
/// let contents = loader.load("minimap/metadata_terrain_desert_seashore.dds").unwrap();
/// // Process the file...
///
/// ```
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

/// Fetches the CDN URL for the given patch or game version of PoE
///
/// # Examples
/// ```
/// let cache_path = PathBuf::from(".cache");
/// // Latest version of PoE 1
/// let url = cdn_base_url(&cache_path, "1").unwrap();
///
/// // Specific patch of PoE 2
/// let url = cdn_base_url(&cache_path, "4.4.0.3.9").unwrap();
/// ```
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
    .unwrap_or_else(|_| panic!("Failed to get URL for version: {}", version));

    fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;
    fs::write(&cache_file, url.as_str()).context("Failed to write URL to cache")?;
    eprintln!("Refreshed CDN URL: {}", url);
    Ok(url)
}

fn parse_response(input: &[u8]) -> IResult<&[u8], Vec<String>> {
    let (input, num_strings) = le_u8(input)?; // Parse the number of strings (N)
    let (input, _) = take(33usize)(input)?; // Discard the next 33 bytes (padding)
    count(parse_utf16_string, num_strings as usize)(input) // Parse N strings
}

fn parse_utf16_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, len) = le_u8(input)?; // Parse string length (L)
    let (input, utf16_bytes) = take(len as usize * 2)(input)?; // Extract L * 2 bytes of UTF-16 data
    let utf16_words: Vec<u16> = utf16_bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    let string =
        String::from_utf16(&utf16_words).expect("Failed to parse response as UTF-16 string");

    Ok((input, string))
}

/// Fetch the current latest version of the game
fn cur_url(host: String, send: &[u8]) -> anyhow::Result<Url> {
    // Fetch data from the CDN
    // TODO: looks like this returns a list of URLs. Might need to use a
    // streaming-style parsing instead of just reading 1Kb down the line if there's a bunch of
    // strings
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(send)?;
    let mut buf = [0; 1024];
    let read = stream.read(&mut buf)?;

    // Parse the response
    let strings = if let Ok((_, strings)) = parse_response(&buf[..read]) {
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
