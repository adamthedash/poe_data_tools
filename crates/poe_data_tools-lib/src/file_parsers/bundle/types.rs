use serde::Serialize;

/// Encoded as a u32
#[derive(Debug, Clone, Serialize)]
pub enum FirstFileEncode {
    Kraken6,    // 8
    MermaidA,   // 9
    Bitknit,    // 12
    LeviathanC, // 13
}

#[derive(Debug, Clone, Serialize)]
pub struct HeadPayload {
    pub unk1: [u8; 12],
    pub first_file_encode: FirstFileEncode,
    pub unk2: [u8; 4],
    pub uncompressed_size: u64,
    pub total_payload_size: u64,
    pub block_count: u32,
    pub uncompressed_block_granularity: u32,
    pub unk3: [u8; 16],
}

#[derive(Debug, Clone, Serialize)]
pub struct BundleFile {
    pub head: HeadPayload,
    pub blocks: Vec<Vec<u8>>,
}
