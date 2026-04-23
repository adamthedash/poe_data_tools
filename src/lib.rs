#![feature(trait_alias)]
#![feature(str_from_utf16_endian)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(never_type)]
#![feature(f16)]
#![feature(try_find)]
#![feature(option_reference_flattening)]
use std::sync::OnceLock;

pub mod bundle;
pub mod commands;
pub mod dat;
pub mod file_parsers;
pub mod fs;
pub mod hasher;
pub mod path;
pub mod tree;

/// Application-level verbosity
pub static VERBOSE: OnceLock<bool> = OnceLock::new();
