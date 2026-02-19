use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TDTFile {
    pub version: u32,
    pub strings: Vec<String>,
    pub strings_offsets: Vec<usize>,
    pub nums1: Vec<u32>,
    // pub num2: u32,
    pub nums3: Vec<Option<u32>>,
    pub nums3_string: Vec<Option<String>>,
    // pub nums4: Vec<u8>,
    pub rest: Vec<u8>,
}
