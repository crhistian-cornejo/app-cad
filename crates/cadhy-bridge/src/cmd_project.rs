//! Project-level commands: new, undo, redo, save, load

use tauri::State;
use tracing::info;

use crate::dto::UndoStateDto;
use crate::error::BridgeResult;
use crate::state::AppState;

/// Simple ping command to check if the plugin is available
#[tauri::command]
pub async fn project_ping() -> BridgeResult<bool> {
    info!("project_ping called");
    Ok(true)
}

/// Create a new empty project
#[tauri::command]
pub async fn project_new(state: State<'_, AppState>) -> BridgeResult<()> {
    // Clear scene
    {
        let mut scene = state
            .scene
            .write()
            .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;
        scene.clear();
    }

    // Clear command history
    {
        let mut commands = state
            .commands
            .write()
            .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;
        commands.clear();
    }

    state.mark_dirty();
    Ok(())
}

/// Undo the last command
#[tauri::command]
pub async fn project_undo(state: State<'_, AppState>) -> BridgeResult<UndoStateDto> {
    // We can't call undo directly because CommandContext requires scene access
    // For now, just report the state - real undo needs refactoring
    let commands = state
        .commands
        .read()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    // TODO: Implement actual undo when CommandContext is properly integrated
    // This requires accessing both scene and commands simultaneously

    Ok(UndoStateDto {
        can_undo: commands.can_undo(),
        can_redo: commands.can_redo(),
        undo_name: commands.undo_name().map(String::from),
        redo_name: commands.redo_name().map(String::from),
        is_modified: commands.is_modified(),
    })
}

/// Redo the last undone command
#[tauri::command]
pub async fn project_redo(state: State<'_, AppState>) -> BridgeResult<UndoStateDto> {
    let commands = state
        .commands
        .read()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;

    // TODO: Implement actual redo when CommandContext is properly integrated

    Ok(UndoStateDto {
        can_undo: commands.can_undo(),
        can_redo: commands.can_redo(),
        undo_name: commands.undo_name().map(String::from),
        redo_name: commands.redo_name().map(String::from),
        is_modified: commands.is_modified(),
    })
}

/// Check if undo is available
#[tauri::command]
pub async fn project_can_undo(state: State<'_, AppState>) -> BridgeResult<bool> {
    let commands = state
        .commands
        .read()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;
    Ok(commands.can_undo())
}

/// Check if redo is available
#[tauri::command]
pub async fn project_can_redo(state: State<'_, AppState>) -> BridgeResult<bool> {
    let commands = state
        .commands
        .read()
        .map_err(|e| crate::error::BridgeError::StateLock(e.to_string()))?;
    Ok(commands.can_redo())
}
