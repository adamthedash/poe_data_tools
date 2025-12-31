//! High level commands for working with virtual files

pub mod cat;
pub mod dump_art;
pub mod dump_tables;
pub mod extract;
pub mod list;

#[derive(Debug, Clone)]
pub enum Patch {
    One,
    Two,
    Specific(String),
}

impl std::str::FromStr for Patch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Patch::One),
            "2" => Ok(Patch::Two),
            _ => Ok(Patch::Specific(s.to_string())),
        }
    }
}
