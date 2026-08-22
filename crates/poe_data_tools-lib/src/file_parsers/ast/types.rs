use serde::Serialize;
use serde_with::serde_as;

use crate::file_parsers::bundle::types::BundleFile;

#[derive(Clone, Debug, Serialize)]
pub struct ASTFile {
    pub header: Header,
    pub bones: Vec<Bone>,
    pub lights: Vec<Light>,
    pub animations: Vec<Animation>,
    pub bundle: Option<BundleFile>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Header {
    pub version: u8,
    pub num_bones: u8,
    pub unk1: u8,
    pub num_animations: u16,
    pub unk3: u8,
    pub unk4: u8,
    pub num_lights: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct Bone {
    pub sibling: Option<u8>,
    pub child: Option<u8>,
    pub transform: [[f32; 4]; 4],
    pub name_length: u8,
    pub unk1: Option<u8>,
    pub name: String,
}

#[serde_as]
#[derive(Clone, Debug, Serialize)]
pub struct Light {
    pub name_length: u8,
    #[serde_as(as = "[_; _]")]
    pub unk_bytes1: [u8; 51],
    pub unk_bytes2: Option<[u8; 4]>,
    pub unk_bytes3: Option<[u8; 4]>,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DataLocation {
    pub offset: u32,
    pub length: u32,
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
    pub data_location: Option<DataLocation>,
    pub name: String,
    pub parent_name: Option<String>,
    pub data: Option<Vec<Track>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrackHeader {
    pub unk1: u8,
    pub index: u32,
    pub num_scales: u32,
    pub num_rotations: u32,
    pub num_positions: u32,
    pub num_unk2: u32,
    pub num_unk3: u32,
    pub num_unk4: u32,
    pub unk5: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Track {
    pub header: TrackHeader,

    pub scales: Vec<[f32; 4]>,
    pub rotations: Vec<[f32; 5]>,
    pub positions: Vec<[f32; 4]>,
    pub unk2s: Vec<[f32; 4]>,
    pub unk3s: Vec<[f32; 5]>,
    pub unk4s: Vec<[f32; 4]>,
}
