use serde::Serialize;

use crate::file_parsers::dolm::types::{Dolm, DolmVertex, IndexBuffer};

#[derive(Debug, Clone, Serialize)]
pub struct TGMFile {
    pub version: u8,
    pub bbox: [f32; 6],
}

#[derive(Debug, Clone, Serialize)]
pub struct ShapeExtentsV8 {
    pub ordinal: Option<u16>,
    pub bbox: [f32; 6],
    pub index_base: u32,
    pub index_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Mesh {
    pub shape_extents: Vec<ShapeExtentsV8>,
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
    pub extra_u16: u16,
    pub extra_u8: u8,
    pub geometries: Vec<Geometry>,
    pub tail: Vec<TailEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TailEntry {
    pub uint1: u32,
    pub floats: [f32; 12],
    pub uint2: u32,
    pub bytes: [u8; 31],
}

#[derive(Debug, Clone, Serialize)]
pub struct ShapeExtentsV9 {
    pub ordinal: u32,
    pub bbox: [f32; 6],
}

#[derive(Debug, Clone, Serialize)]
pub struct Geometry {
    pub dolm: Dolm,
    pub shape_extents: Vec<ShapeExtentsV9>,
}
