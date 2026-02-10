use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Line1 {
    pub scale: f32,
    pub uint1: Option<u32>,
    pub uint2: Option<u32>,
}

#[derive(Debug, Serialize)]
pub enum Weight {
    Float(f32),
    All,
}

#[derive(Debug, Serialize)]
pub struct Group {
    pub name: String,
    pub d: Option<String>,
    pub float1: Option<f32>,
    pub objects: Vec<Object>,
}

#[derive(Debug, Serialize)]
pub struct Object {
    pub weight: Weight,
    pub ao_file: String,
    pub uint1: Option<u32>,
    pub d: Option<String>,
    pub float1: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct DDTFile {
    pub version: u32,
    pub line1: Line1,
    pub uint1: Option<u32>,
    pub groups: Vec<Group>,
}
