use std::sync::OnceLock;

pub mod commands;
pub mod file_parser;

/// Application-level verbosity
pub static VERBOSE: OnceLock<bool> = OnceLock::new();
