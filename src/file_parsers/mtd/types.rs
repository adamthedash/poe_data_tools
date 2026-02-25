use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MTDFile {
    pub version: u32,
    pub groups: Vec<Group>,
}

#[derive(Debug, Serialize)]
pub struct Group {
    pub name: Option<String>,
    pub entries: Vec<Entry>,
    pub weight_line: Option<(Vec<u32>, u32)>,
    pub extra_line: Option<(u32, bool)>,
    pub extra_entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub mat_file: String,
    pub dlp_files: Vec<String>,
}
