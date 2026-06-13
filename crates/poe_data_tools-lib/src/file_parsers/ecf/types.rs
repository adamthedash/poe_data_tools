use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EcfFile {
    pub version: u32,
    pub combinations: Vec<EcfCombination>,
}

#[derive(Debug, Serialize)]
pub struct EcfCombination {
    pub et_files: [Option<String>; 3],
    pub uint1: Option<u32>,
}
