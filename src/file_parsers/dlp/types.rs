use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Entry {
    pub fmt_file: String,
    pub float: f32,
    pub points: Vec<(f32, f32)>,
}

#[derive(Debug, Serialize)]
pub struct Nums {
    pub float1: f32,
    pub float2: f32,
    pub bool1: bool,
    pub bool2: bool,
    pub uint: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct DLPFile {
    pub version: Option<u32>,
    pub nums: Nums,
    pub entries: Vec<Entry>,
}
