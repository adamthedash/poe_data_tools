use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Header {
    pub tdt_file: Option<(String, Vec<u8>)>,
    pub common_tgt: Option<String>,
    pub tag: Option<String>,
    pub side_ets: [Option<String>; 4],
    pub dimensions: [u8; 2],
    pub side_gts: [Option<String>; 4],
    pub dimensions2: Option<[u8; 2]>,
    pub extra_nums: [u8; 2],
    pub side_offsets: [u8; 8],
}

#[derive(Debug, Serialize)]
pub struct Subtile {
    pub kind: u8,
    pub value: u8,
    pub string_index: u8,
    pub fixed_block: Option<Vec<u8>>,
    pub variable_block: Option<Vec<u8>>,
    // pub string: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TDTFile {
    pub version: u32,
    pub strings: HashMap<usize, Option<String>>,
    pub header: Header,
    pub flags: [u8; 8],
    pub bytes: [u8; 3],
    pub num_subtiles: u16,
    pub subtiles: Vec<Subtile>,
    pub rest: Vec<u8>,
    pub dims3: [[u8; 4]; 2],
    pub trailing_uint3: u8,
    pub trailing_uint4: u8,
    pub trailing_uint2: Option<u8>,
    pub trailing_uint1: Option<u8>,
    // pub trailing: Option<Vec<u8>> // len == prod(hedaer.dimensions) if it is present
}
