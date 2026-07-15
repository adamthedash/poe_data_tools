use std::sync::OnceLock;

pub mod commands;
pub mod file_parser;
pub mod tree;

/// Application-level verbosity
pub static VERBOSE: OnceLock<bool> = OnceLock::new();
