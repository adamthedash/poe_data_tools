pub mod cat;
pub mod dump_arm;
pub mod dump_art;
pub mod dump_ecf;
pub mod dump_rs;
pub mod dump_tables;
pub mod dump_trees;
pub mod dump_tsi;
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
