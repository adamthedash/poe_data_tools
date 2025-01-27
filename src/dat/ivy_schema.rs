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

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::ivy_schema::SchemaCollection;

    #[test]
    fn load_schema() {
        let schema: SchemaCollection =
            serde_json::from_reader(File::open("schema.json").unwrap()).unwrap();
        println!("{:#?}", schema);
    }
}
