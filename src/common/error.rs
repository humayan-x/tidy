use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum TidyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Path resolution error: {0}")]
    PathResolution(String),

    #[error("Directory traversal error: {0}")]
    Traversal(String),

    #[error("File operation error at '{path}': {message}")]
    FileOperation { path: PathBuf, message: String },

    #[error("Watcher error: {0}")]
    Watcher(String),

    #[error("Service error: {0}")]
    Service(String),

    #[error("Operation aborted: {0}")]
    Aborted(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, TidyError>;
