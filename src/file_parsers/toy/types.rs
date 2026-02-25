use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Rotation {
    pub flip: bool,
    pub angle: u32,
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub weight: u32,
    pub arm_file: String,
    pub key_values: Vec<(String, String)>,
    pub not_flags: Vec<String>,
    pub add_flags: Vec<String>,
    pub rotations: Vec<Rotation>,
}

#[derive(Debug, Serialize, Copy, Clone)]
pub enum Order {
    File,
    Size,
}

#[derive(Debug, Serialize)]
pub struct Header {
    pub bool1: bool,
    pub bool2: bool,
    pub file_order: Option<Order>,
    pub flags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Group {
    pub header: Header,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
pub struct TOYFile {
    pub version: u32,
    pub groups: Vec<Group>,
}
