//! Command stack for undo/redo functionality
//!
//! This implements a traditional undo/redo stack with support for:
//! - Undo limit (memory management)
//! - Command merging (for continuous operations)
//! - Save point tracking (for "modified" indicator)

use tracing::{debug, trace, warn};

use crate::command::{Command, CommandContext, CommandResult};
use crate::error::{CommandError, Result};

/// Maximum number of commands to keep in history
const DEFAULT_UNDO_LIMIT: usize = 100;

/// Manages the undo/redo stack
///
/// The stack maintains two vectors:
/// - `undo_stack`: Commands that can be undone (most recent last)
/// - `redo_stack`: Commands that can be redone (most recent last)
///
/// When a new command is executed, the redo stack is cleared.
#[derive(Debug)]
pub struct CommandStack {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    undo_limit: usize,
    save_point: Option<usize>, // Index in undo_stack when last saved
}

impl Default for CommandStack {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandStack {
    /// Create a new empty command stack
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_limit: DEFAULT_UNDO_LIMIT,
            save_point: Some(0),
        }
    }

    /// Create with a custom undo limit
    pub fn with_limit(limit: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_limit: limit,
            save_point: Some(0),
        }
    }

    /// Execute a command and add it to the undo stack
    pub fn execute(
        &mut self,
        mut command: Box<dyn Command>,
        ctx: &mut dyn CommandContext,
    ) -> CommandResult {
        debug!("Executing command: {}", command.name());

        // Execute the command
        let result = command.execute(ctx)?;

        // Clear redo stack when new command is executed
        if !self.redo_stack.is_empty() {
            trace!("Clearing redo stack ({} commands)", self.redo_stack.len());
            self.redo_stack.clear();
        }

        // Check if we should merge with previous command
        if command.merge_with_previous() && !self.undo_stack.is_empty() {
            if let Some(last) = self.undo_stack.last_mut() {
                if last.try_merge(command.as_ref()) {
                    trace!("Merged command with previous: {}", last.name());
                    return Ok(result);
                }
            }
        }

        // Add to undo stack if undoable
        if command.is_undoable() {
            self.undo_stack.push(command);
            self.enforce_limit();
        } else {
            trace!("Command not undoable, not adding to stack");
        }

        Ok(result)
    }

    /// Undo the last command
    pub fn undo(&mut self, ctx: &mut dyn CommandContext) -> Result<()> {
        let mut command = self
            .undo_stack
            .pop()
            .ok_or(CommandError::NothingToUndo)?;

        debug!("Undoing command: {}", command.name());
        command.undo(ctx)?;

        self.redo_stack.push(command);
        Ok(())
    }

    /// Redo the last undone command
    pub fn redo(&mut self, ctx: &mut dyn CommandContext) -> CommandResult {
        let mut command = self
            .redo_stack
            .pop()
            .ok_or(CommandError::NothingToRedo)?;

        debug!("Redoing command: {}", command.name());
        let result = command.redo(ctx)?;

        self.undo_stack.push(command);
        Ok(result)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the name of the command that would be undone
    pub fn undo_name(&self) -> Option<&str> {
        self.undo_stack.last().map(|c| c.name())
    }

    /// Get the name of the command that would be redone
    pub fn redo_name(&self) -> Option<&str> {
        self.redo_stack.last().map(|c| c.name())
    }

    /// Get the number of commands in the undo stack
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of commands in the redo stack
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Mark current state as saved
    ///
    /// This is used to track whether the document has been modified.
    pub fn mark_saved(&mut self) {
        self.save_point = Some(self.undo_stack.len());
        debug!("Marked save point at position {}", self.undo_stack.len());
    }

    /// Check if document has been modified since last save
    pub fn is_modified(&self) -> bool {
        match self.save_point {
            Some(point) => self.undo_stack.len() != point,
            None => true, // No save point = always modified
        }
    }

    /// Clear all command history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.save_point = None;
        debug!("Command stack cleared");
    }

    /// Enforce the undo limit by removing oldest commands
    fn enforce_limit(&mut self) {
        while self.undo_stack.len() > self.undo_limit {
            self.undo_stack.remove(0);

            // Adjust save point if it was affected
            if let Some(point) = self.save_point {
                if point == 0 {
                    // Save point was removed
                    self.save_point = None;
                    warn!("Save point removed due to undo limit");
                } else {
                    self.save_point = Some(point - 1);
                }
            }
        }
    }

    /// Get undo history as list of command names (most recent first)
    pub fn undo_history(&self) -> Vec<&str> {
        self.undo_stack.iter().rev().map(|c| c.name()).collect()
    }

    /// Get redo history as list of command names (most recent first)
    pub fn redo_history(&self) -> Vec<&str> {
        self.redo_stack.iter().rev().map(|c| c.name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    // Mock context for testing
    struct MockContext;

    impl CommandContext for MockContext {
        fn scene(&self) -> &dyn Any {
            self
        }
        fn scene_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn viewport(&self) -> Option<&dyn Any> {
            None
        }
        fn viewport_mut(&mut self) -> Option<&mut dyn Any> {
            None
        }
        fn mark_dirty(&mut self) {}
    }

    // Test command that increments/decrements a counter
    #[derive(Debug)]
    struct IncrementCommand {
        counter: Arc<AtomicI32>,
        amount: i32,
    }

    impl Command for IncrementCommand {
        fn name(&self) -> &str {
            "Increment"
        }

        fn execute(&mut self, _ctx: &mut dyn CommandContext) -> CommandResult {
            self.counter.fetch_add(self.amount, Ordering::SeqCst);
            Ok(None)
        }

        fn undo(&mut self, _ctx: &mut dyn CommandContext) -> Result<()> {
            self.counter.fetch_sub(self.amount, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_execute_and_undo() {
        let mut stack = CommandStack::new();
        let mut ctx = MockContext;
        let counter = Arc::new(AtomicI32::new(0));

        // Execute
        let cmd = Box::new(IncrementCommand {
            counter: counter.clone(),
            amount: 5,
        });
        stack.execute(cmd, &mut ctx).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 5);
        assert!(stack.can_undo());
        assert!(!stack.can_redo());

        // Undo
        stack.undo(&mut ctx).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 0);
        assert!(!stack.can_undo());
        assert!(stack.can_redo());

        // Redo
        stack.redo(&mut ctx).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 5);
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_modified_tracking() {
        let mut stack = CommandStack::new();
        let mut ctx = MockContext;
        let counter = Arc::new(AtomicI32::new(0));

        // Initially not modified
        assert!(!stack.is_modified());

        // Execute makes it modified
        let cmd = Box::new(IncrementCommand {
            counter: counter.clone(),
            amount: 1,
        });
        stack.execute(cmd, &mut ctx).unwrap();
        assert!(stack.is_modified());

        // Mark saved
        stack.mark_saved();
        assert!(!stack.is_modified());

        // Undo makes it modified again
        stack.undo(&mut ctx).unwrap();
        assert!(stack.is_modified());

        // Redo brings us back to save point
        stack.redo(&mut ctx).unwrap();
        assert!(!stack.is_modified());
    }
}
