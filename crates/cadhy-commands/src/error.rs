//! Command error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Undo failed: {0}")]
    UndoFailed(String),

    #[error("Redo failed: {0}")]
    RedoFailed(String),

    #[error("No commands to undo")]
    NothingToUndo,

    #[error("No commands to redo")]
    NothingToRedo,

    #[error("Command cannot be undone")]
    NotUndoable,

    #[error("Invalid object ID: {0}")]
    InvalidObjectId(uuid::Uuid),

    #[error("Scene error: {0}")]
    SceneError(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, CommandError>;
