//! CADHY Commands - Command pattern implementation for undo/redo
//!
//! This crate implements the Command pattern used throughout CADHY for:
//! - Undo/Redo functionality
//! - Action history
//! - Macro recording
//!
//! Architecture inspired by Blender's ED_undo system.

mod command;
mod error;
mod stack;

pub use command::{Command, CommandContext, CommandResult};
pub use error::{CommandError, Result};
pub use stack::CommandStack;

/// Re-export common types
pub use glam;
pub use uuid::Uuid;
