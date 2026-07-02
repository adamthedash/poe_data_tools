use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Entry {
    pub weight: Option<u32>,
    pub tdt_file: String,
    pub rotations: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TSTFile {
    pub includes: Vec<String>,
    pub tdt_files: Vec<Entry>,
}
