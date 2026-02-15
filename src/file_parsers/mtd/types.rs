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
    pub nums: Option<(u32, u32)>,
    pub extra_mat_files: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub mat_file: String,
    pub dlp_file: Option<String>,
}
