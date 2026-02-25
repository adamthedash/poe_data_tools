use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Item {
    pub uint1: u32,
    pub stub: String,
    pub ao_file: Option<String>,
    pub float1: f32,
    pub float2: Option<f32>,
    pub uint2: u32,
    pub uint3: u32,
}

#[derive(Debug, Serialize)]
pub struct Group {
    pub name: String,
    pub float: Option<f32>,
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize)]
pub struct CLTFile {
    pub version: u32,
    pub float1: f32,
    pub groups: Vec<Group>,
}
