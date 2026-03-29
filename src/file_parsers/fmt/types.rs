use serde::{Serialize, Serializer};
use serde_with::{SerializeAs, serde_as};

use super::dolm::types::*;

/// For serlializing f16 types since serde doesn't implement it natively
pub(super) struct SerF16;

impl SerializeAs<f16> for SerF16 {
    fn serialize_as<S>(source: &f16, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f32(*source as f32)
    }
}

#[derive(Debug, Serialize)]
pub struct FMTFile {
    pub version: u8,
    pub bbox: [f32; 6],

    pub section: Section,
    pub shapes: Vec<Shape>,

    pub d0s: Vec<[u8; 6]>,
    pub d1s: Vec<[u8; 12]>,
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
pub enum IndexBuffer {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

#[derive(Debug, Serialize)]
pub struct Shape {
    pub name: String,
    pub material: String,
    pub triangle_start: u32,
}
