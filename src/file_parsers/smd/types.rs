use serde::Serialize;

use crate::file_parsers::dolm::types::{Dolm, DolmVertex, IndexBuffer};

#[derive(Debug, Serialize)]
pub struct SMDFile {
    pub version: u8,
    pub vertex_format: u8,
    pub section: Section,

    pub bbox: [f32; 6],
    pub tail: Tail,
}

#[derive(Debug, Serialize)]
pub enum Section {
    V2(V2Section),
    V3(V3Section),
}

#[derive(Debug, Serialize)]
pub struct V3Section {
    pub dolm: Dolm,
    pub shape_names: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ShapeExtents {
    pub name: String,
    pub triangle_index: u32,
}

#[derive(Debug, Serialize)]
pub struct V2Section {
    pub c04_2: Option<u32>,
    pub shape_extents: Vec<ShapeExtents>,
    pub index_buffer: IndexBuffer,
    pub vertex_buffer: Vec<DolmVertex>,
}

#[derive(Debug, Serialize)]
pub struct Ellipsoid {
    pub floats: [f32; 15],
    pub unk1: u32,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Sphere {
    pub centre: [f32; 3],
    pub radius: f32,
    pub unk1: u32,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SphereConnection {
    pub s0_index: u32,
    pub s1_index: u32,
}

#[derive(Debug, Serialize)]
pub struct SkinnedVertex {
    pub pos: [f32; 3],
    pub unk1: [u32; 4],
    pub unk2: [f32; 4],
}

#[derive(Debug, Serialize)]
pub struct Tail {
    pub tail_version: u32,
    pub ellipsoids: Vec<Ellipsoid>,
    pub spheres: Vec<Sphere>,
    pub sphere_connections: Vec<SphereConnection>,
    pub skinned_vertices: Vec<SkinnedVertex>,
    pub t3s: Vec<()>,
    pub sv_refs1: Vec<u32>,
    pub sv_refs2: Vec<u32>,
}
