use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GTFile {
    pub name: String,
    pub bool1: bool,
    pub bool2: bool,
    pub bool3: Option<bool>,
    pub bool4: Option<bool>,
    pub bool5: Option<bool>,
    pub string1: Option<String>,
}
