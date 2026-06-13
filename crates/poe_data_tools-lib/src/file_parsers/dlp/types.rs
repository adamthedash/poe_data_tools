use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Entry {
    pub fmt_file: String,
    pub float: f32,
    pub points: Vec<(f32, f32)>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", content = "data")]
pub enum Header {
    RandomScale { min: f32, max: f32 },
    AllowWaving,
    AllowOnBlocking,
    MaxRotation(u32),
    MinEdgeScale(f32),
    AudioType(u32),
    DelayMultiplier(f32),
    SizeMultiplier(f32),
    TimeMultiplier(f32),
    Seed(u32),
    Other { key: String, rest: String },
}

#[derive(Debug, Serialize)]
pub struct HeadersV3(pub Vec<Header>);

#[derive(Debug, Serialize)]
pub struct HeadersV2 {
    pub scale_min: f32,
    pub scale_max: f32,
    pub allow_waving: bool,
    pub allow_on_blocking: bool,
    pub max_rotation: Option<u32>,
    pub uint1: Option<u32>,
    pub uint2: Option<u32>,
    pub float1: Option<f32>,
    pub audio_type: Option<u32>,
    pub float2: Option<f32>,
}

#[derive(Debug, Serialize)]
pub enum Headers {
    V2(HeadersV2),
    V3(HeadersV3),
}

#[derive(Debug, Serialize)]
pub struct DLPFile {
    pub version: Option<u32>,
    pub headers: Headers,
    pub entries: Vec<Entry>,
}
