//! CADHY Bridge - Tauri IPC layer between React UI and Rust core
//!
//! This crate provides the communication bridge for:
//! - Tauri commands (React → Rust)
//! - Events (Rust → React)
//! - State management
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │          React UI (Webview)         │
//! │   Toolbar, Panels, Properties, AI   │
//! └─────────────────┬───────────────────┘
//!                   │ Tauri IPC
//!                   ▼
//! ┌─────────────────────────────────────┐
//! │         cadhy-bridge (this)         │
//! │  viewport_*, scene_*, cad_*, etc.   │
//! └─────────────────┬───────────────────┘
//!                   │
//!       ┌───────────┼───────────┐
//!       ▼           ▼           ▼
//! ┌──────────┐ ┌──────────┐ ┌──────────┐
//! │ viewport │ │ commands │ │  kernel  │
//! │  (wgpu)  │ │(undo/redo)│ │  (OCCT)  │
//! └──────────┘ └──────────┘ └──────────┘
//! ```
//!
//! # Command Naming Convention
//!
//! - `viewport_*` - Viewport control (camera, render mode)
//! - `scene_*` - Scene manipulation (add, remove, select)
//! - `cad_*` - CAD operations (primitives, boolean)
//! - `project_*` - Project management (save, load, undo)
//! - `hydraulics_*` - Hydraulic calculations

mod dto;
mod error;
mod state;

pub mod cmd_project;
pub mod cmd_scene;
pub mod cmd_viewport;

pub use dto::*;
pub use error::{BridgeError, BridgeResult};
pub use state::AppState;

use tauri::Manager;

/// Initialize the Tauri plugin with all commands
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("cadhy-bridge")
        .invoke_handler(tauri::generate_handler![
            // Project commands
            cmd_project::project_ping,
            cmd_project::project_new,
            cmd_project::project_undo,
            cmd_project::project_redo,
            cmd_project::project_can_undo,
            cmd_project::project_can_redo,
            // Viewport commands
            cmd_viewport::viewport_init,
            cmd_viewport::viewport_is_initialized,
            cmd_viewport::viewport_render_frame,
            cmd_viewport::viewport_resize,
            cmd_viewport::viewport_get_size,
            cmd_viewport::viewport_set_camera,
            cmd_viewport::viewport_get_camera,
            cmd_viewport::viewport_orbit,
            cmd_viewport::viewport_pan,
            cmd_viewport::viewport_zoom,
            cmd_viewport::viewport_frame_selection,
            cmd_viewport::viewport_frame_all,
            cmd_viewport::viewport_set_view_mode,
            cmd_viewport::viewport_get_view_mode,
            // Scene commands
            cmd_scene::scene_get_objects,
            cmd_scene::scene_add_object,
            cmd_scene::scene_add_primitive,
            cmd_scene::scene_add_cube,
            cmd_scene::scene_remove_object,
            cmd_scene::scene_select,
            cmd_scene::scene_deselect_all,
            cmd_scene::scene_get_selection,
            cmd_scene::scene_set_transform,
            cmd_scene::scene_get_transform,
        ])
        .setup(|app, _api| {
            // Initialize app state
            app.manage(AppState::new());
            Ok(())
        })
        .build()
}
