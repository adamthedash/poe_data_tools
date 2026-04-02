use serde::Serialize;

use crate::file_parsers::dolm::types::Dolm;

#[derive(Debug, Serialize)]
pub struct SMDFile {
    version: u8,
    bbox: [f32; 6],
    shapes: Vec<String>,
    dolm: Dolm,
    tail_version: u32,
    tail: Tail,
}

#[derive(Debug, Serialize)]
pub struct Tail {
    ellipsoids: Vec<Ellipsoid>,
    spheres: Vec<Sphere>,
    t2s: Vec<[u32; 2]>,
    t3s: Vec<()>,
    t4s: Vec<SkinnedVertex>,
    t5s: Vec<u32>,
    t6s: Vec<u32>,
}

#[derive(Debug, Serialize)]
pub struct Ellipsoid {}

#[derive(Debug, Serialize)]
pub struct Sphere {}

#[derive(Debug, Serialize)]
pub struct SkinnedVertex {}
