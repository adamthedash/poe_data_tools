use std::collections::HashMap;

use serde::Serialize;

pub type Emitter = HashMap<String, String>;

#[derive(Debug, Serialize, Clone)]
pub struct TRLFile {
    pub version: Option<u32>,
    pub emitters: Vec<Emitter>,
    pub payload: Option<serde_json::Value>,
}
