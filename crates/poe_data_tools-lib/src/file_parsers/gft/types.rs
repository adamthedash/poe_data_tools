use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct GenFile {
    pub weight: u32,
    pub path: String,
    pub rotations: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Section {
    pub name: String,
    pub uint1: Option<u32>,
    /// .arm or .tdt
    pub files: Vec<GenFile>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GFTFile {
    pub version: u32,
    pub sections: Vec<Section>,
}
