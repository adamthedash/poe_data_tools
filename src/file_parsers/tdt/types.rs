use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Header {
    pub unk_string: Option<String>,
    pub common_tgt: Option<String>,
    pub tag: Option<String>,
    pub side_ets: [Option<String>; 4],
    pub dimensions: [u8; 2],
    pub side_gts: [Option<String>; 4],
    pub dimensions2: Option<[u8; 2]>,
    pub side_offsets: [u8; 8],
}

#[derive(Debug, Serialize)]
pub struct TDTFile {
    pub version: u32,
    pub strings: HashMap<usize, Option<String>>,
    pub header: Header,
    pub rest: Vec<u8>,
}
