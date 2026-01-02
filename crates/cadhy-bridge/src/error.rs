//! Bridge error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("Viewport error: {0}")]
    Viewport(String),

    #[error("Viewport initialization error: {0}")]
    ViewportInit(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("Scene error: {0}")]
    Scene(String),

    #[error("Command error: {0}")]
    Command(#[from] cadhy_commands::CommandError),

    #[error("Object not found: {0}")]
    ObjectNotFound(uuid::Uuid),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("State lock error: {0}")]
    StateLock(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Implement Serialize for Tauri IPC
impl serde::Serialize for BridgeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type BridgeResult<T> = Result<T, BridgeError>;
