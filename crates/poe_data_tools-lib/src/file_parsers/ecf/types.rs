use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct EcfFile {
    pub version: u32,
    pub combinations: Vec<EcfCombination>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EcfCombination {
    pub et_files: [Option<String>; 3],
    pub uint1: Option<u32>,
}
