use serde::Serialize;

use crate::file_parsers::dolm::types::{Dolm, DolmVertex, IndexBuffer};

#[derive(Debug, Clone, Serialize)]
pub struct TGMFile {
    pub version: u8,
    pub bbox: [f32; 6],
}

#[derive(Debug, Clone, Serialize)]
pub struct ShapeExtents {
    pub ordinal: Option<u16>,
    pub bbox: [f32; 6],
    pub index_base: u32,
    pub index_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Mesh {
    pub shape_extents: Vec<ShapeExtents>,
    pub indices: IndexBuffer,
    pub vertices: Vec<DolmVertex>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Section {
    V8(V8Section),
    V9(V9Section),
}

#[derive(Debug, Clone, Serialize)]
pub struct V8Section {
    pub vertex_format: u8,
    pub extra_header: Option<u32>,
    pub meshes: Vec<Mesh>,
    pub tail_entries: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct V9Section {
    pub extra_bytes: [u8; 4],
    pub geometries: Vec<Geometry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShapeExtentsV9 {
    pub ordinal: u32,
    pub main_bytes: Option<[u8; 2]>,
    pub bbox: [f32; 6],
}

#[derive(Debug, Clone, Serialize)]
pub struct Geometry {
    pub dolm: Dolm,
    pub shape_extents: Vec<ShapeExtentsV9>,
}
