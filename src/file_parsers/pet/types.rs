use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Emitter {
    pub emitter_type: String,
    pub material: Option<String>,
    pub key_values: String,
}

#[derive(Debug, Serialize)]
pub struct PETFile {
    pub version: Option<u32>,
    pub emitters: Vec<Emitter>,
    pub payload: Option<serde_json::Value>,
}
