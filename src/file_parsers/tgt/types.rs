use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TGTFile {
    pub version: u32,
    pub size: [u32; 2],
    pub tile_mesh_root: String,
    pub ground_mask: Option<String>,
    pub normal_materials: Vec<String>,
    pub material_slots: Option<Vec<String>>,
    pub subtile_material_indices: Option<Vec<Vec<Vec<Index>>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Index {
    pub uint1: u32,
    pub uint2: u32,
    pub uint3: Option<u32>,
}
