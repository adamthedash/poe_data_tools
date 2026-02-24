use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GcfFile {
    pub version: u32,
    pub combinations: Vec<GcfCombination>,
}

#[derive(Debug, Serialize)]
pub struct GcfCombination {
    pub gt_files: [String; 3],
}
