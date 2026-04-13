use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SMFile {
    pub version: u32,
    pub smd_file: String,
    pub materials: Vec<Material>,
    pub bbox: Option<[f32; 6]>,
    pub bone_groups: Option<Vec<BoneGroup>>,
}

#[derive(Debug, Serialize)]
pub struct Material {
    pub mat_file: Option<String>,
    pub unk1: u32,
}

#[derive(Debug, Serialize)]
pub struct BoneGroup {
    pub name: String,
    pub bones: Vec<String>,
}
