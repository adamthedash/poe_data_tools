#![feature(trait_alias)]
#![feature(str_from_utf16_endian)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(never_type)]
#![feature(f16)]
#![feature(try_find)]
#![feature(option_reference_flattening)]
#![feature(try_blocks)]

pub mod bundle;
pub mod dat;
pub mod file_parsers;
pub mod fs;
pub mod hasher;
pub mod path;
pub mod tree;

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
