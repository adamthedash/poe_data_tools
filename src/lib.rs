use std::sync::OnceLock;

pub mod bundle;
pub mod bundle_fs;
pub mod bundle_index;
pub mod bundle_loader;
pub mod commands;
pub mod dat;
pub mod hasher;
pub mod path;
pub mod steam;

/// Application-level verbosity
pub static VERBOSE: OnceLock<bool> = OnceLock::new();

pub use bundle::{fetch_bundle_content, load_bundle_content, parse_bundle, Bundle};
pub use bundle_fs::FS as FileSystem;
pub use bundle_index::{fetch_index_file, load_index_file, parse_bundle_index, BundleIndex};
pub use bundle_loader::{cdn_base_url, CDNLoader};
pub use dat::{
    ivy_schema::{fetch_schema, SchemaCollection},
    table_view::DatTable,
};
