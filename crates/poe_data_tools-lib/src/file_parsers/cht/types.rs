use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Entry {
    pub weight: u32,
    pub chest_types: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Nums {
    pub float1: f32,
    pub float2: f32,
    pub uint1: u32,
    pub uint2: u32,
    pub uint3: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub enum NumLine {
    V2(f32),
    V3(Nums),
}

#[derive(Debug, Serialize, Clone)]
pub struct Group {
    pub areas: Vec<String>,
    pub nums: Option<NumLine>,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CHTFile {
    pub version: u32,
    pub groups: Vec<Group>,
}
