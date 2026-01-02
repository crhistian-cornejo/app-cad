//! I/O error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IoError {
    #[error("File I/O error: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, IoError>;
