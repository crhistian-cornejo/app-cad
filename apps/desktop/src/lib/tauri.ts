import { invoke } from "@tauri-apps/api/core"

// Types
export interface ViewportSettings {
  show_grid: boolean
  show_axes: boolean
  anti_aliasing: boolean
}

// Viewport API - Supports both native (60+ FPS) and fallback modes
export const viewportApi = {
  // === NATIVE MODE (High Performance) ===

  /** Initialize native viewport with a dedicated wgpu window */
  initNative: (width: number, height: number, x: number, y: number) =>
    invoke<boolean>("plugin:cadhy-bridge|viewport_init_native", { width, height, x, y }),

  /** Check if native mode is active */
  isNative: () =>
    invoke<boolean>("plugin:cadhy-bridge|viewport_is_native"),

  /** Update native viewport position and size */
  updateBounds: (x: number, y: number, width: number, height: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_update_bounds", { x, y, width, height }),

  /** Orbit camera (native mode) */
  orbitNative: (deltaX: number, deltaY: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_orbit_native", { delta_x: deltaX, delta_y: deltaY }),

  /** Pan camera (native mode) */
  panNative: (deltaX: number, deltaY: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_pan_native", { delta_x: deltaX, delta_y: deltaY }),

  /** Zoom camera (native mode) */
  zoomNative: (delta: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_zoom_native", { delta }),

  // === FALLBACK MODE (Compatibility) ===

  init: (width: number, height: number) =>
    invoke<boolean>("plugin:cadhy-bridge|viewport_init", { width, height }),

  resize: (width: number, height: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_resize", { width, height }),

  renderFrame: () =>
    invoke<string>("plugin:cadhy-bridge|viewport_render_frame"),

  orbit: (deltaX: number, deltaY: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_orbit", {
      input: { delta_x: deltaX, delta_y: deltaY },
    }),

  pan: (deltaX: number, deltaY: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_pan", {
      input: { delta_x: deltaX, delta_y: deltaY },
    }),

  zoom: (delta: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_zoom", { input: { delta } }),

  // === SHARED COMMANDS ===

  frameAll: () =>
    invoke<void>("plugin:cadhy-bridge|viewport_frame_all"),

  resetCamera: () =>
    invoke<void>("plugin:cadhy-bridge|viewport_reset_camera"),

  setViewMode: (mode: "solid" | "wireframe") =>
    invoke<void>("plugin:cadhy-bridge|viewport_set_view_mode", { mode }),

  getViewMode: () =>
    invoke<"solid" | "wireframe">("plugin:cadhy-bridge|viewport_get_view_mode"),

  setSettings: (settings: ViewportSettings) =>
    invoke<void>("plugin:cadhy-bridge|viewport_set_settings", { settings }),

  getSettings: () =>
    invoke<ViewportSettings>("plugin:cadhy-bridge|viewport_get_settings"),

  isDirty: () =>
    invoke<boolean>("plugin:cadhy-bridge|viewport_is_dirty"),

  clearDirty: () =>
    invoke<void>("plugin:cadhy-bridge|viewport_clear_dirty"),
}

// Scene API
export const sceneApi = {
  addCube: (name: string, size: number) =>
    invoke<string>("plugin:cadhy-bridge|scene_add_cube", { name, size }),
}

// Project API
export const projectApi = {
  newProject: () =>
    invoke<void>("plugin:cadhy-bridge|project_new"),

  undo: () =>
    invoke<void>("plugin:cadhy-bridge|project_undo"),

  redo: () =>
    invoke<void>("plugin:cadhy-bridge|project_redo"),

  canUndo: () =>
    invoke<boolean>("plugin:cadhy-bridge|project_can_undo"),

  canRedo: () =>
    invoke<boolean>("plugin:cadhy-bridge|project_can_redo"),
}
