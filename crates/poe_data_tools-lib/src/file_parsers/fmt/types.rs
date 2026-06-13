use serde::Serialize;
use serde_with::serde_as;

use crate::file_parsers::{
    dolm::types::{Dolm, IndexBuffer},
    shared::serialise::SerF16,
};

#[derive(Debug, Serialize)]
pub struct Subcomponent {
    pub unk1: u8,
    pub d1s: Vec<[u8; 12]>,
    pub tag: String,
}

#[derive(Debug, Serialize)]
pub struct FMTFile {
    pub version: u8,
    pub bbox: [f32; 6],

    pub section: Section,
    pub shapes: Vec<Shape>,

    pub subcomponents: Vec<Subcomponent>,
    pub d3s: Vec<Vec<u8>>,
    pub string_table: String,
}

#[derive(Debug, Serialize)]
pub enum Section {
    V8(V8Section),
    V9(Dolm),
}

#[derive(Debug, Serialize)]
pub struct V8Section {
    pub vertex_format: Option<u32>,
    pub index_buffer: IndexBuffer,
    pub vertex_buffer: Vec<Vertex>,
}

#[serde_as]
#[derive(Debug, Serialize)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub unk: [u8; 8],
    #[serde_as(as = "[SerF16; _]")]
    pub uv: [f16; 2],
    #[serde_as(as = "Option<[SerF16; _]>")]
    pub uv2: Option<[f16; 2]>,
}

#[derive(Debug, Serialize)]
pub struct Shape {
    pub name: String,
    pub material: String,
    pub triangle_start: u32,
}
