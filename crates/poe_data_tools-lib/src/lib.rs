#![feature(trait_alias)]
#![feature(f16)]
#![feature(option_reference_flattening)]

mod bundle;
pub mod dat;
pub mod file_parsers;
pub mod fs;
pub mod hasher;
mod path;

/// The version of the Path of Exile game
#[derive(Debug, Clone)]
pub enum Patch {
    /// Any version of PoE 1
    One,
    /// Any version of PoE 2
    Two,
    /// A specific game patch eg. "3.4.0.12"
    /// PoE1 - "3.*"
    /// PoE2 - "4.*"
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
            Specific(s) => panic!("Invalid major patch version {s:?}"),
        }
    }

    pub fn as_str(&self) -> &str {
        use Patch::*;
        match self {
            One => "1",
            Two => "2",
            Specific(v) => v,
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
