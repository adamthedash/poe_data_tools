use std::sync::OnceLock;

pub mod commands;

/// Application-level verbosity
pub static VERBOSE: OnceLock<bool> = OnceLock::new();
