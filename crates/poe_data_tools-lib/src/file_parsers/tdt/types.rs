use serde::Serialize;
use serde_with::serde_as;

#[derive(Debug, Clone, Serialize)]
pub struct TDTFile {
    pub version: u32,
    pub strings: String,
    pub header: Header,
    pub tail: Option<Tail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Header {
    pub tdt_file: Option<String>,
    pub flags: Option<u8>,
    pub num: Option<u16>,
    pub tgt_file: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tail {
    pub side_ets: [Option<String>; 4],
    pub dims: [u8; 2],
    pub side_gts: [Option<String>; 4],
    pub dims2: [u8; 2],
    pub dims3: [u8; 2],
    pub side_offsets: [u8; 8],
    pub flags1: u8,
    pub flags2: u8,
    pub flags_rest: Vec<u8>,
    pub subtiles: Vec<Subtile>,
    pub trailing: Trailing,
}

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct Subtile {
    pub kind: u8,
    pub value: u8,
    pub bytes: Vec<u8>,
    #[serde_as(as = "Option<[_; _]>")]
    pub fixed_block: Option<FixedBlock>,
    pub vary_block: Option<Vec<u8>>,
}

pub type FixedBlock = [u8; 23 * 23];

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct Trailing {
    pub flags1_1_strings: Option<Vec<Option<String>>>,
    pub flags1_0_bools: Option<[bool; 4]>,
    pub strings: [Option<String>; 2],
    pub tail_bools: [bool; 3],
    // pub flags1_2_bools: Option<[bool; 2]>,
    pub flags1_3_string: Option<String>,
    #[serde_as(as = "Option<Vec<[_; _]>>")]
    pub flags1_4_blocks: Option<Vec<FixedBlock>>,
    pub flags1_5_floats: Option<Vec<f32>>,
}
