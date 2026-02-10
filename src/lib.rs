#![feature(trait_alias)]
#![feature(str_from_utf16_endian)]
#![feature(assert_matches)]
#![feature(closure_lifetime_binder)]
use std::sync::OnceLock;

pub mod bundle;
pub mod bundle_fs;
pub mod bundle_index;
pub mod bundle_loader;
pub mod commands;
pub mod dat;
pub mod file_parsers;
pub mod hasher;
pub mod path;
pub mod steam;
pub mod tree;

/// Application-level verbosity
pub static VERBOSE: OnceLock<bool> = OnceLock::new();
