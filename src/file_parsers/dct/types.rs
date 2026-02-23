use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Entry {
    pub weight: u32,
    pub atlas_file: String,
    pub tag: String,
    pub float: f32,
}

#[derive(Debug, Serialize)]
pub struct Group {
    pub area: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
pub struct DCTFile {
    pub version: u32,
    pub float: f32,
    pub groups: Vec<Group>,
}
