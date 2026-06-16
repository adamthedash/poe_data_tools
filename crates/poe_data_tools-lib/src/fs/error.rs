use std::sync::Arc;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors for the file system
// NOTE: Need to wrap non-clone variants in Arc so we can return the same error
// for all files when a batch read fails
#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    /// Any issue related to reading/writing files
    #[error(transparent)]
    IO(#[from] Arc<std::io::Error>),

    /// Any issue bubbled up by reqwest
    #[error(transparent)]
    Reqwest(#[from] Arc<reqwest::Error>),

    /// Catchall for any failures when processing network responses
    #[error("received invalid response: {0}")]
    InvalidResponse(String),

    /// Bad user-provided config value catchall
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// Path does not exist in the virtual file system
    #[error("file not found in virtual file system: {0:?}")]
    FileNotFound(String),

    /// Issue with interpreting bytes as structured data
    // TODO: Get rid of anyhow once we have proper parser errors
    #[error("failed to parse data")]
    Parse(#[from] Arc<anyhow::Error>),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Arc::new(value).into()
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Arc::new(value).into()
    }
}
