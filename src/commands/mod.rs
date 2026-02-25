pub mod cat;
pub mod dump_art;
pub mod dump_tables;
pub mod dump_trees;
pub mod extract;
pub mod list;
pub mod translate;

#[derive(Debug, Clone)]
pub enum Patch {
    One,
    Two,
    Specific(String),
}

impl Patch {
    /// PoE 1 or 2
    pub fn major(&self) -> u32 {
        use Patch::*;
        match self {
            One => 1,
            Two => 2,
            Specific(s) if s.starts_with("3.") => 1,
            Specific(s) if s.starts_with("4.") => 1,
            Specific(s) => unreachable!("Invalid major patch version {s:?}"),
        }
    }
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
