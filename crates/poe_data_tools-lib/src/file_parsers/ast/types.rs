use serde::Serialize;
use serde_with::serde_as;

use crate::file_parsers::bundle::types::BundleFile;

#[derive(Clone, Debug, Serialize)]
pub struct ASTFile {
    pub header: Header,
    pub bones: Vec<Bone>,
    pub lights: Vec<Light>,
    pub animations: Vec<Animation>,
    pub bundle: BundleFile,
}

#[derive(Clone, Debug, Serialize)]
pub struct Header {
    pub version: u8,
    pub num_bones: u8,
    pub unk1: u8,
    pub num_animations: u8,
    pub unk2: u8,
    pub unk3: u8,
    pub unk4: u8,
    pub num_lights: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct Bone {
    pub sibling: u8,
    pub child: u8,
    pub transform: [[f32; 4]; 4],
    pub name_length: u8,
    pub unk1: u8,
    pub name: String,
}

#[serde_as]
#[derive(Clone, Debug, Serialize)]
pub struct Light {
    pub name_length: u8,
    #[serde_as(as = "[_; _]")]
    pub unk_bytes1: [u8; 55],
    pub unk_bytes2: Option<[u8; 4]>,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Animation {
    pub num_tracks: u8,
    pub unk1: u8,
    pub framerate: u8,
    pub unk2: u8,
    pub unk3: Option<u8>,
    pub name_length: u8,
    pub parent_name_length: Option<u8>,
    pub data_offset: u32,
    pub data_size: u32,
    pub name: String,
    pub parent_name: Option<String>,
}
