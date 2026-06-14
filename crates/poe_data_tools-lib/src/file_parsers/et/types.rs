use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NumLine {
    pub uint1: u32,
    pub uint2: u32,
    pub bool1: bool,
    pub bool2: Option<bool>,
    pub bool3: Option<bool>,
    pub bool4: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct VirtualETFile {
    pub path: String,
    pub bool1: bool,
}

#[derive(Debug, Serialize)]
pub struct VirtualSection {
    pub virtual_et_files: [VirtualETFile; 2],
    pub virtual_rotations: [u32; 2],
}

#[derive(Debug, Serialize)]
pub enum GTFile {
    Wildcard,
    Path(String),
}

#[derive(Debug, Serialize)]
pub struct ETFile {
    pub name: String,
    pub hex: Option<String>,
    pub gt_files: [GTFile; 2],
    pub num_line: Option<NumLine>,
    pub gt_file2: Option<String>,
    pub virtual_section: Option<VirtualSection>,
}
