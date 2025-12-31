//! Dat file schema loading using [community curated](https://github.com/poe-tool-dev/dat-schema)
//! source of truth

use std::{fs, path::Path};

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SchemaCollection {
    pub tables: Vec<DatTableSchema>,
}

#[derive(Deserialize, Debug)]
pub struct DatTableSchema {
    /// PoE Version
    /// 1 - PoE 1
    /// 2 - PoE 2
    /// 3 - Common
    #[serde(rename = "validFor")]
    pub valid_for: u32,
    /// Table name
    pub name: String,
    pub columns: Vec<ColumnSchema>,
}

#[derive(Deserialize, Debug)]
pub struct ColumnSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub array: bool,
    #[serde(rename = "type")]
    pub column_type: String,
    pub unique: bool,
    pub localized: bool,
    pub references: Option<References>,
    pub until: Option<String>,
    pub file: Option<String>,
    pub files: Option<Vec<String>>,
    pub interval: bool,
}

#[derive(Deserialize, Debug)]
pub struct References {
    pub table: String,
}

pub fn fetch_schema(cache_dir: &Path) -> Result<SchemaCollection> {
    const SCHEMA_URL: &str =
        "https://github.com/poe-tool-dev/dat-schema/releases/download/latest/schema.min.json";

    let cache_dir = cache_dir.join("schema");
    let schema_path = cache_dir.join("schema.min.json");
    let etag_path = schema_path.with_extension("json.etag");

    // File fresh? Use it
    if let Ok(metadata) = fs::metadata(&schema_path) {
        if metadata.modified()?.elapsed()?.as_secs() < 3600 {
            eprintln!("Using cached schema");
            return Ok(serde_json::from_str(
                fs::read_to_string(schema_path)?.as_str(),
            )?);
        }
    }

    eprintln!("Fetching schema from github");
    let client = reqwest::blocking::Client::new();
    let mut req = client.get(SCHEMA_URL);

    // Got an etag? Use it
    if let Ok(etag) = fs::read_to_string(&etag_path) {
        req = req.header("If-None-Match", etag);
    }

    let response = req.send()?.error_for_status()?;
    if response.status().as_u16() == 304 {
        // Not modified
        let schema_file = fs::read_to_string(&schema_path)?;
        return Ok(serde_json::from_str(&schema_file)?);
    }

    // Save out to cache
    fs::create_dir_all(cache_dir)?;
    fs::write(
        etag_path,
        response.headers().get("etag").unwrap().to_str().unwrap(),
    )?;
    let content = response.bytes()?;
    fs::write(&schema_path, content)?;

    Ok(serde_json::from_str(
        fs::read_to_string(schema_path)?.as_str(),
    )?)
}

#[cfg(test)]
mod tests {
    use dirs::cache_dir;

    use crate::dat::ivy_schema::fetch_schema;

    #[test]
    fn load_schema() {
        let cache_dir = cache_dir().unwrap().join("poe_data_tools");
        let schema = fetch_schema(&cache_dir).unwrap();
        println!("{:#?}", schema);
    }
}
