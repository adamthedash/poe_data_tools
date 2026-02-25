/// Encoded as a u32
#[derive(Debug)]
pub enum FirstFileEncode {
    Kraken6,    // 8
    MermaidA,   // 9
    Bitknit,    // 12
    LeviathanC, // 13
}

#[derive(Debug)]
pub struct HeadPayload {
    pub first_file_encode: FirstFileEncode,
    pub uncompressed_size: u64,
    pub total_payload_size: u64,
    pub uncompressed_block_granularity: u32,
}

#[derive(Debug)]
pub struct BundleFile {
    pub head: HeadPayload,
    pub blocks: Vec<Vec<u8>>,
}
