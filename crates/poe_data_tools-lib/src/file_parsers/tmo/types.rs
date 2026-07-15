use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Override {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TMOFile {
    pub version: u32,
    pub overrides: Vec<Override>,
}
