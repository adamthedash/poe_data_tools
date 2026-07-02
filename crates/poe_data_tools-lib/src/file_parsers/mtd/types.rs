use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct MTDFile {
    pub version: u32,
    pub groups: Vec<Group>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Group {
    pub name: Option<String>,
    pub entries: Vec<Entry>,
    pub weight_line: Option<(Vec<u32>, u32)>,
    pub extra_line: Option<(u32, bool)>,
    pub extra_entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Entry {
    pub mat_file: String,
    pub dlp_files: Vec<String>,
}
