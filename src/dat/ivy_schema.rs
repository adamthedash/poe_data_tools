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

pub fn fetch_schema(cache_dir: &std::path::Path) -> anyhow::Result<SchemaCollection> {
    const SCHEMA_URL: &str =
        "https://github.com/poe-tool-dev/dat-schema/releases/download/latest/schema.min.json";
    let schema_path = cache_dir.join("schema.min.json");
    let etag_path = schema_path.with_extension("json.etag");
    let client = reqwest::blocking::Client::new();

    if etag_path.exists() && schema_path.exists() {
        // check if the schema file was modified more than an hour ago and just return it if not
        let metadata = std::fs::metadata(&schema_path)?;
        if metadata.modified()?.elapsed()?.as_secs() < 3600 {
            let schema: SchemaCollection =
                serde_json::from_reader(std::fs::File::open(schema_path)?)?;
            return Ok(schema);
        }

        // check if the etag has changed
        let etag = std::fs::read_to_string(&etag_path)?;
        let response = client
            .head(SCHEMA_URL)
            .header("If-None-Match", etag)
            .send()?
            .error_for_status()?;
        if response.status().as_u16() == 304 {
            let schema: SchemaCollection =
                serde_json::from_reader(std::fs::File::open(schema_path)?)?;
            return Ok(schema);
        }
    }

    let response = client.get(SCHEMA_URL).send()?.error_for_status()?;
    let etag = response.headers().get("etag").unwrap().to_str().unwrap();
    std::fs::write(etag_path, etag)?;
    let content = response.bytes()?;
    std::fs::create_dir_all(cache_dir)?;
    std::fs::write(&schema_path, content)?;
    let schema: SchemaCollection = serde_json::from_reader(std::fs::File::open(schema_path)?)?;
    Ok(schema)
}

#[cfg(test)]
mod tests {
    use crate::dat::ivy_schema::fetch_schema;
    use dirs::cache_dir;

    #[test]
    fn load_schema() {
        let cache_dir = cache_dir().unwrap().join("poe_data_tools");
        let schema = fetch_schema(&cache_dir).unwrap();
        println!("{:#?}", schema);
    }
}
