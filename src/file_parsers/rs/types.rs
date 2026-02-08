use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RSFile {
    pub version: u32,
    pub rooms: Vec<Room>,
}

#[derive(Debug, Serialize)]
pub struct Room {
    pub weight: Option<u32>,
    pub arm_file: String,
    pub rotations: Vec<String>,
}
