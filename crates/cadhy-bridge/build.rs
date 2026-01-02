// Build script for cadhy-bridge plugin
// This generates permissions for all Tauri commands

const COMMANDS: &[&str] = &[
    // Project commands
    "project_ping",
    "project_new",
    "project_undo",
    "project_redo",
    "project_can_undo",
    "project_can_redo",
    // Viewport commands (fallback mode)
    "viewport_init",
    "viewport_is_initialized",
    "viewport_render_frame",
    "viewport_resize",
    "viewport_get_size",
    "viewport_set_camera",
    "viewport_get_camera",
    "viewport_orbit",
    "viewport_pan",
    "viewport_zoom",
    "viewport_frame_selection",
    "viewport_frame_all",
    "viewport_set_view_mode",
    "viewport_get_view_mode",
    "viewport_reset_camera",
    "viewport_set_settings",
    "viewport_get_settings",
    "viewport_is_dirty",
    "viewport_clear_dirty",
    // Native viewport commands (high-performance mode)
    "viewport_init_native",
    "viewport_is_native",
    "viewport_update_bounds",
    "viewport_orbit_native",
    "viewport_pan_native",
    "viewport_zoom_native",
    // Scene commands
    "scene_get_objects",
    "scene_add_object",
    "scene_add_primitive",
    "scene_add_cube",
    "scene_remove_object",
    "scene_select",
    "scene_deselect_all",
    "scene_get_selection",
    "scene_set_transform",
    "scene_get_transform",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .build();
}
