use std::{
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

use anyhow::{Error, Ok};
use url::Url;

pub struct Loader {
    base_url: Url,
    cache_dir: String,
}

impl Loader {
    pub fn new(version: &str, cache_dir: &str) -> Self {
        // 1 == latest for PoE 1
        // 2 == latest for PoE 2
        // 4.* == specific version for PoE 2
        // * == specific version for PoE 1
        if version == "1" {
            let base_url = cur_url_poe().unwrap();
            return Self {
                base_url,
                cache_dir: cache_dir.to_string(),
            };
        }
        if version == "2" {
            let base_url = cur_url_poe2().unwrap();
            return Self {
                base_url,
                cache_dir: cache_dir.to_string(),
            };
        }
        let base_url = if version.starts_with("4") {
            Url::parse(cur_url_poe2().unwrap().as_str()).unwrap()
        } else {
            Url::parse(cur_url_poe().unwrap().as_str()).unwrap()
        };
        Self {
            base_url,
            cache_dir: cache_dir.to_string(),
        }
    }

    pub fn load(&self, path: &Path) -> anyhow::Result<Vec<u8>> {
        let url = self.base_url.join(path.to_str().unwrap())?;
        let client = reqwest::blocking::Client::new();
        let path = std::path::PathBuf::from(&self.cache_dir)
            .join(url.to_string().trim_start_matches("https://"));
        if path.exists() {
            let req = client.get(url.clone());
            let res = req.send()?;
            if res.status().is_success() {
                let etag = res.headers().get("ETag").unwrap().to_str().unwrap();
                let etag_path = path.with_extension("etag");
                let file_etag = std::fs::read_to_string(&etag_path).unwrap_or_default();
                if file_etag == etag {
                    println!("Using cached file: {}", path.display());
                    let mut file = std::fs::File::open(&path)?;
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                    return Ok(buffer);
                }
            }
        }

        println!("Downloading file: {}", url);
        let req = client.get(url.clone());
        let res = req.send()?;

        if !res.status().is_success() {
            return Err(Error::msg(format!(
                "Failed to download {}: {}",
                url,
                res.status()
            )));
        }
        std::fs::create_dir_all(path.parent().unwrap())?;
        {
            let etag = res.headers().get("ETag").unwrap().to_str().unwrap();
            let etag_path = path.with_extension("etag");
            let mut etag_file = std::fs::File::create(etag_path)?;
            etag_file.write_all(etag.as_bytes())?;
        }
        let bytes = res.bytes()?;
        let mut file = std::fs::File::create(&path)?;
        file.write_all(&bytes)?;
        Ok(bytes.to_vec())
    }
}

fn cur_url_poe() -> anyhow::Result<Url> {
    cur_url("patch.pathofexile.com:12995".to_string(), &[1, 6])
}
fn cur_url_poe2() -> anyhow::Result<Url> {
    cur_url("patch.pathofexile2.com:13060".to_string(), &[1, 7])
}

fn cur_url(host: String, send: &[u8]) -> anyhow::Result<Url> {
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(send)?;
    let mut buf = [0; 1024];
    let read = stream.read(&mut buf)?;
    assert!(read > 33);
    let mut data = &buf[34..read];
    let len = data[0] as usize;
    data = &data[1..];
    if len > data.len() {
        return Err(Error::msg("Invalid length"));
    }
    let raw = data
        .chunks(2)
        .take(len)
        .map(|c| u16::from_le_bytes(c.try_into().unwrap()))
        .collect::<Vec<_>>();
    let s = String::from_utf16(&raw)?;
    let url = Url::parse(&s)?;
    Ok(url)
}
