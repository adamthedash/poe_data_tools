use serde::Serialize;
use serde_with::serde_as;

use super::super::types::*;

#[serde_as]
#[derive(Debug, Serialize)]
pub struct Dolm {
    pub c0h: u16,
    pub vertex_format: u32,
    pub lod_extents: Vec<[u32; 2]>,
    pub lods: Vec<Mesh>,

    #[serde_as(as = "Option<Vec<[_; _]>>")]
    pub extra_vformat_6: Option<Vec<[u8; 36]>>,
    pub extra_vformat_6_c0h_2: Option<Vec<[u8; 4]>>,
    pub extra_c0h_4: Option<[u8; 4]>,
}

#[derive(Debug, Serialize)]
pub struct Mesh {
    pub shape_extents: Vec<DolmShapeExtents>,
    pub indices: IndexBuffer,
    pub vertices: Vec<DolmVertex>,
}

#[derive(Debug, Serialize)]
pub struct DolmShapeExtents {
    pub start_index: u32,
    pub count_index: u32,
}

#[serde_as]
#[derive(Debug, Serialize)]
pub struct DolmVertex {
    pub pos: [f32; 3],
    pub normal: [i8; 4],
    pub tangent: [i8; 4],
    #[serde_as(as = "Option<[SerF16; _]>")]
    pub tex_coord0: Option<[f16; 2]>,
    #[serde_as(as = "Option<[SerF16; _]>")]
    pub tail_float: Option<[f16; 2]>,
    pub skin_bones: Option<[u8; 4]>,
    pub skin_weights: Option<[u8; 4]>,
    pub skin_extra: Option<[u8; 4]>,
    #[serde_as(as = "Option<[SerF16; _]>")]
    pub tex_coord1: Option<[f16; 2]>,
    pub extra_vformat_6: Option<[u8; 4]>,
}
