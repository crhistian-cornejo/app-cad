//! Command trait and context
//!
//! The Command pattern is fundamental to CAD applications.
//! Every user action that modifies state is encapsulated as a Command.

use std::any::Any;
use std::fmt::Debug;

use crate::error::Result;

/// Result of command execution containing data to send back to UI
pub type CommandResult = Result<Option<Box<dyn Any + Send + Sync>>>;

/// Context provided to commands during execution
///
/// This provides access to the application state without commands
/// needing to know the concrete types (which live in cadhy-bridge).
pub trait CommandContext: Send + Sync {
    /// Get the scene graph (read-only)
    fn scene(&self) -> &dyn Any;

    /// Get the scene graph (mutable)
    fn scene_mut(&mut self) -> &mut dyn Any;

    /// Get the viewport renderer (read-only)
    fn viewport(&self) -> Option<&dyn Any>;

    /// Get the viewport renderer (mutable)
    fn viewport_mut(&mut self) -> Option<&mut dyn Any>;

    /// Mark scene as modified (triggers re-render)
    fn mark_dirty(&mut self);
}

/// A command that can be executed, undone, and redone
///
/// Commands are the fundamental unit of user interaction.
/// They encapsulate all information needed to:
/// 1. Execute an action
/// 2. Undo that action
/// 3. Redo the action
///
/// # Design Notes
///
/// - Commands should be serializable for macro recording
/// - Commands should be small and focused (single responsibility)
/// - Complex operations should be composed of multiple commands
pub trait Command: Debug + Send + Sync {
    /// Human-readable name for the undo stack
    fn name(&self) -> &str;

    /// Execute the command
    ///
    /// Returns any data that should be sent back to the UI.
    fn execute(&mut self, ctx: &mut dyn CommandContext) -> CommandResult;

    /// Undo the command
    ///
    /// This should restore the exact state before `execute` was called.
    fn undo(&mut self, ctx: &mut dyn CommandContext) -> Result<()>;

    /// Redo the command
    ///
    /// Default implementation just calls execute again.
    /// Override if redo requires different logic.
    fn redo(&mut self, ctx: &mut dyn CommandContext) -> CommandResult {
        self.execute(ctx)
    }

    /// Whether this command can be undone
    ///
    /// Some commands (like "save file") cannot be undone.
    fn is_undoable(&self) -> bool {
        true
    }

    /// Whether this command should be merged with the previous command
    ///
    /// Useful for continuous operations like dragging, where many small
    /// movements should be merged into a single undo step.
    fn merge_with_previous(&self) -> bool {
        false
    }

    /// Try to merge with another command of the same type
    ///
    /// Returns true if merge was successful.
    /// This is used for continuous operations like transform dragging.
    fn try_merge(&mut self, _other: &dyn Command) -> bool {
        false
    }
}

/// A command that groups multiple commands together
#[derive(Debug)]
pub struct CompositeCommand {
    name: String,
    commands: Vec<Box<dyn Command>>,
}

impl CompositeCommand {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            commands: Vec::new(),
        }
    }

    pub fn add(&mut self, command: impl Command + 'static) {
        self.commands.push(Box::new(command));
    }

    pub fn with(mut self, command: impl Command + 'static) -> Self {
        self.add(command);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Command for CompositeCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&mut self, ctx: &mut dyn CommandContext) -> CommandResult {
        for cmd in &mut self.commands {
            cmd.execute(ctx)?;
        }
        Ok(None)
    }

    fn undo(&mut self, ctx: &mut dyn CommandContext) -> Result<()> {
        // Undo in reverse order
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(ctx)?;
        }
        Ok(())
    }

    fn is_undoable(&self) -> bool {
        self.commands.iter().all(|c| c.is_undoable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCommand {
        executed: bool,
        value: i32,
        previous_value: Option<i32>,
    }

    impl TestCommand {
        fn new(value: i32) -> Self {
            Self {
                executed: false,
                value,
                previous_value: None,
            }
        }
    }

    impl Command for TestCommand {
        fn name(&self) -> &str {
            "Test Command"
        }

        fn execute(&mut self, _ctx: &mut dyn CommandContext) -> CommandResult {
            self.executed = true;
            Ok(None)
        }

        fn undo(&mut self, _ctx: &mut dyn CommandContext) -> Result<()> {
            self.executed = false;
            Ok(())
        }
    }
}
