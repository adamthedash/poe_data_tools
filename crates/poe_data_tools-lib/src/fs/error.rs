use std::sync::Arc;

use crate::fs::cdn::CDNError;

pub type Result<T, E = FSError> = std::result::Result<T, E>;

/// Errors for the file system
// NOTE: Need to wrap non-clone variants in Arc so we can return the same error
// for all files when a batch read fails
#[derive(Debug, thiserror::Error, Clone)]
pub enum FSError {
    /// Any issue related to reading/writing files
    #[error(transparent)]
    IO(#[from] Arc<std::io::Error>),

    /// Any issue bubbled up by reqwest
    #[error(transparent)]
    Reqwest(#[from] Arc<reqwest::Error>),

    /// Failures related to CDN downloader
    #[error("error during CDN download")]
    CDNError(#[from] CDNError),

    /// Bad user-provided config value catchall
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// Path does not exist in the virtual file system
    #[error("file not found in virtual file system: {0:?}")]
    FileNotFound(String),

    /// Issue with interpreting bytes as structured data
    #[error(transparent)]
    Parse(#[from] Arc<crate::file_parsers::error::ParseError>),
}

impl From<std::io::Error> for FSError {
    fn from(value: std::io::Error) -> Self {
        Arc::new(value).into()
    }
}

impl From<reqwest::Error> for FSError {
    fn from(value: reqwest::Error) -> Self {
        Arc::new(value).into()
    }
}

impl From<crate::file_parsers::error::ParseError> for FSError {
    fn from(value: crate::file_parsers::error::ParseError) -> Self {
        Arc::new(value).into()
    }
}
