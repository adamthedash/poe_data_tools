use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AnimationStage {
    pub name: String,
    pub uint1: u32,
    pub floats: [f32; 3],
}

#[derive(Debug, Serialize)]
pub struct BoneGroup {
    pub name: String,
    pub bones: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BoneRotation {
    pub bone: String,
    pub coord_order: String,
    pub coords: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct Group {
    pub name: String,
    pub animation_type: String,
    pub animation_time: u32,
    pub animation_stages: Vec<AnimationStage>,
    pub float_group: Vec<f32>,
    pub bone_rotations: Option<Vec<BoneRotation>>,
    pub extra_ints: Option<(i32, i32)>,
}

#[derive(Debug, Serialize)]
pub struct AMDFile {
    pub version: u32,
    pub groups: Vec<Group>,
    pub bone_groups: Option<Vec<BoneGroup>>,
}
