use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Entry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Struct {
    pub name: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AOFile {
    pub version: u32,
    pub is_abstract: bool,
    pub extends: Vec<String>,
    pub structs: Vec<Struct>,
    pub client_structs: Vec<Struct>,
}
