use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Header {
    // Weird conditional stuff
    pub tdt_file: Option<String>,
    pub flags: Option<u8>,
    pub num1: Option<u16>,
    pub tgt_tmd_file: Option<String>,
    pub tag: Option<String>,

    // Consistent
    pub side_ets: [Option<String>; 4],
    pub dimensions: [u8; 2],
    pub side_gts: [Option<String>; 4],
    pub dimensions2: Option<[u8; 2]>,
    pub extra_nums: [u8; 2],
    pub side_offsets: [u8; 8],

    // Inconsistent
    pub flags1: u8,
    pub flags2: u8,
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
    pub num_tiles_offset: usize,
    pub rest: Vec<u8>,
    // pub flags: [u8; 8],
    // pub bytes: [u8; 3],
    // pub num_subtiles: u16,

    // pub subtiles: Vec<Subtile>,
    // pub dims3: [[u8; 4]; 2],
    // pub trailing_uint3: u8,
    // pub trailing_uint4: u8,
    // pub trailing_uint2: Option<u8>,
    // pub trailing_uint1: Option<u8>,
    // pub trailing: Option<Vec<u8>> // len == prod(hedaer.dimensions) if it is present
}
