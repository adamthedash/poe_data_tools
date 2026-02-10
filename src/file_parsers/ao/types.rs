use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Entry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Struct {
    pub name: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
pub struct AOFile {
    pub version: u32,
    pub is_abstract: bool,
    pub extends: Vec<String>,
    pub structs: Vec<Struct>,
}
