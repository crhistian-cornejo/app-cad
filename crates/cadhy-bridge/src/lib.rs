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

pub mod cmd_cad;
pub mod cmd_overlay;
pub mod cmd_project;
pub mod cmd_scene;
pub mod cmd_viewport;
pub mod embedded_viewport;
pub mod ui_overlay;
pub mod wgpu_overlay;

pub use cmd_overlay::OverlayViewportState;
pub use cmd_viewport::WgpuOverlayState;
pub use dto::*;
pub use embedded_viewport::{EmbeddedViewport, FpsStats, RenderMessage, RenderSender};
pub use error::{BridgeError, BridgeResult};
pub use state::AppState;
pub use ui_overlay::{OverlayLayout, UiOverlay, ViewportBounds};
pub use wgpu_overlay::WgpuOverlay;

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
            cmd_viewport::viewport_reset_camera,
            cmd_viewport::viewport_set_settings,
            cmd_viewport::viewport_get_settings,
            cmd_viewport::viewport_is_dirty,
            cmd_viewport::viewport_clear_dirty,
            // Native viewport commands (high-performance mode)
            cmd_viewport::viewport_init_native,
            cmd_viewport::viewport_is_native,
            cmd_viewport::viewport_update_bounds,
            cmd_viewport::viewport_orbit_native,
            cmd_viewport::viewport_pan_native,
            cmd_viewport::viewport_zoom_native,
            cmd_viewport::viewport_get_fps,
            // WgpuOverlay commands (direct WebviewWindow surface rendering)
            cmd_viewport::viewport_init_overlay,
            cmd_viewport::viewport_overlay_active,
            cmd_viewport::viewport_overlay_resize,
            cmd_viewport::viewport_overlay_orbit,
            cmd_viewport::viewport_overlay_pan,
            cmd_viewport::viewport_overlay_zoom,
            cmd_viewport::viewport_overlay_reset_camera,
            cmd_viewport::viewport_overlay_set_view_mode,
            cmd_viewport::viewport_overlay_set_camera,
            cmd_viewport::viewport_overlay_pick,
            cmd_viewport::viewport_overlay_set_gizmo_mode,
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
            // CAD commands (B-Rep operations)
            cmd_cad::cad_create_primitive,
            cmd_cad::cad_boolean,
            cmd_cad::cad_fuse,
            cmd_cad::cad_cut,
            cmd_cad::cad_common,
            cmd_cad::cad_fillet,
            cmd_cad::cad_chamfer,
            cmd_cad::cad_shell,
            cmd_cad::cad_offset,
            cmd_cad::cad_translate,
            cmd_cad::cad_rotate,
            cmd_cad::cad_mirror,
            cmd_cad::cad_scale,
            // Overlay mode commands (inverted architecture: wgpu main + webview children)
            cmd_overlay::overlay_init,
            cmd_overlay::overlay_get_viewport_bounds,
            cmd_overlay::overlay_set_layout,
            cmd_overlay::overlay_toggle_left_panel,
            cmd_overlay::overlay_toggle_right_panel,
            cmd_overlay::overlay_on_resize,
            cmd_overlay::overlay_is_active,
            cmd_overlay::overlay_camera_orbit,
            cmd_overlay::overlay_camera_pan,
            cmd_overlay::overlay_camera_zoom,
            cmd_overlay::overlay_camera_reset,
        ])
        .setup(|app, _api| {
            // Initialize app state
            app.manage(AppState::new());
            // Initialize overlay viewport state (for inverted architecture mode)
            app.manage(OverlayViewportState::new());
            Ok(())
        })
        .build()
}
