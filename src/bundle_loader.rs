use std::{
    io::{Read, Write},
    net::TcpStream,
};

use anyhow::{Error, Ok};
use url::Url;

pub struct Loader {
    base_url: Url,
    cache_dir: String,
}

impl Loader {
    pub fn new(version: &str, cache_dir: &str) -> Self {
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

    pub fn load(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        let url = self.base_url.join("Bundles2/")?.join(path)?;
        println!("url: {}", url);
        let path = std::path::PathBuf::from(&self.cache_dir)
            .join(url.to_string().trim_start_matches("https://"));
        println!("path2: {:?}", path);
        if path.exists() {
            // todo: check etag
            let mut file = std::fs::File::open(&path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            Ok(buffer)
        } else {
            let response = reqwest::blocking::get(url.clone())?;
            if !response.status().is_success() {
                return Err(Error::msg(format!(
                    "Failed to download {}: {}",
                    url,
                    response.status()
                )));
            }
            let bytes = response.bytes()?;
            std::fs::create_dir_all(path.parent().unwrap())?;
            let mut file = std::fs::File::create(&path)?;
            file.write_all(&bytes)?;
            Ok(bytes.to_vec())
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_index() {
        let loader = Loader::new("1", "cache");
        let result = loader.load("_.index.bin");
        assert!(result.is_ok(), "{}", result.unwrap_err().to_string());
    }
}
