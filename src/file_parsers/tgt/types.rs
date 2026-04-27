use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct V1Section {
    pub tile_mesh: String,
    pub ground_mask: Option<String>,
    pub normal_materials: Vec<V1NormalMaterial>,
}

#[derive(Debug, Clone, Serialize)]
pub struct V1NormalMaterial {
    pub mat_file: String,
    pub uint: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct V3Section {
    pub source_scene: Option<String>,
    pub size: [u32; 2],
    pub tile_mesh_root: String,
    pub ground_mask: Option<String>,
    pub normal_materials: Vec<String>,
    pub material_slots: Option<Vec<String>>,
    pub subtile_material_indices: Option<Vec<Vec<Vec<Index>>>>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Section {
    V1(V1Section),
    V3(V3Section),
}

#[derive(Debug, Clone, Serialize)]
pub struct TGTFile {
    pub version: u32,
    pub section: Section,
}

#[derive(Debug, Clone, Serialize)]
pub struct Index {
    pub uint1: u32,
    pub uint2: u32,
    pub uint3: Option<u32>,
}
