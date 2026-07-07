use std::{fs, path::Path};

use serde::Deserialize;

use crate::Patch;

#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

type Result<T, E = SchemaError> = std::result::Result<T, E>;

/// Full set of Dat table schemas
#[derive(Deserialize, Debug, Clone)]
pub struct SchemaCollection {
    pub tables: Vec<DatTableSchema>,
    pub enumerations: Vec<Enumeration>,
}

impl SchemaCollection {
    /// Filter the schemas for the given game version
    pub fn filter_version(&self, version: &Patch) -> Self {
        let version = version.major();

        Self {
            tables: self
                .tables
                .iter()
                .filter(|t| t.valid_for == version || t.valid_for == 3)
                .cloned()
                .collect(),

            enumerations: self
                .enumerations
                .iter()
                .filter(|e| e.valid_for == version || e.valid_for == 3)
                .cloned()
                .collect(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Enumeration {
    /// PoE Version
    /// 1 - PoE 1
    /// 2 - PoE 2
    /// 3 - Common
    #[serde(rename = "validFor")]
    pub valid_for: u32,
    /// Table name
    pub name: String,
    /// 0 or 1 indexed
    pub indexing: usize,
    /// Enum values
    pub enumerators: Vec<Option<String>>,
}

#[derive(Deserialize, Debug, Clone)]
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

impl DatTableSchema {
    /// Iterate over column names, generating names for unknown columns
    pub fn column_names(&self) -> impl Iterator<Item = String> {
        self.columns.iter().scan(0_usize, |num_unknowns, c| {
            if let Some(name) = &c.name {
                Some(name.clone())
            } else {
                let name = format!("unknown_{}", num_unknowns);
                *num_unknowns += 1;
                Some(name)
            }
        })
    }

    /// Iterate over columns and their names
    pub fn enumerate(&self) -> impl Iterator<Item = (String, &ColumnSchema)> {
        self.column_names().zip(&self.columns)
    }

    /// Iterate over all columns marked unique in the schema
    pub fn primary_keys(&self) -> impl Iterator<Item = String> {
        self.columns
            .iter()
            .zip(self.column_names())
            .filter(|(c, _)| c.unique)
            .map(|(_, name)| name)
    }

    /// Iterate over all reference column names
    pub fn references(&self) -> impl Iterator<Item = String> {
        self.columns
            .iter()
            .filter_map(|c| c.get_ref())
            .map(ToOwned::to_owned)
    }
}

/// Schema for a single column of a table
#[derive(Deserialize, Debug, Clone)]
pub struct ColumnSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    /// Whether this column represents an array of values
    pub array: bool,
    /// Primitive type contained in this column
    #[serde(rename = "type")]
    pub column_type: String,
    /// Whether values in this column are unique (i.e. can be used as a key)
    pub unique: bool,
    pub localized: bool,
    /// Foreign table that values refer to
    pub references: Option<References>,
    pub until: Option<String>,
    pub file: Option<String>,
    pub files: Option<Vec<String>>,
    /// Whether this column represents a numeric interval (from..to)
    pub interval: bool,
}

impl ColumnSchema {
    /// Whether this column refers to another schema
    pub fn is_ref(&self) -> bool {
        matches!(self.column_type.as_str(), "row" | "foreignrow" | "enumrow")
    }

    /// Get the target reference table name, if any
    pub fn get_ref(&self) -> Option<&str> {
        if let Some(r) = &self.references {
            Some(&r.table)
        } else {
            None
        }
    }

    /// Whether this column is a collection of multiple values
    pub fn is_multi(&self) -> bool {
        self.array || self.interval
    }
}

/// Foreign table name
#[derive(Deserialize, Debug, Clone)]
pub struct References {
    pub table: String,
}

/// Load a schema collection from a local path
pub fn load_schema(path: &Path) -> Result<SchemaCollection> {
    log::info!("Loading schema from: {:?}", path);

    let contents = fs::read_to_string(path)?;

    let schema = serde_json::from_str(&contents)?;

    Ok(schema)
}

/// Fetch the latest schema collection or fall back to cache
pub fn fetch_schema(cache_dir: &Path) -> Result<SchemaCollection> {
    const SCHEMA_URL: &str =
        "https://github.com/poe-tool-dev/dat-schema/releases/download/latest/schema.min.json";

    let cache_dir = cache_dir.join("schema");
    let schema_path = cache_dir.join("schema.min.json");
    let etag_path = schema_path.with_extension("json.etag");

    // File fresh? Use it
    if fs::metadata(&schema_path)?
        .modified()?
        .elapsed()
        .map(|d| d.as_secs() < 3600)
        .unwrap_or(false)
    {
        log::info!("Using cached schema");
        return Ok(serde_json::from_str(
            fs::read_to_string(schema_path)?.as_str(),
        )?);
    }

    log::info!("Fetching schema from github");
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
