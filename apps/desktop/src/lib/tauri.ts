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
  isNative: () => invoke<boolean>("plugin:cadhy-bridge|viewport_is_native"),

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

  /** Get FPS statistics from embedded viewport */
  getFps: () =>
    invoke<{ fps: number; frame_time_ms: number }>("plugin:cadhy-bridge|viewport_get_fps"),

  // === FALLBACK MODE (Compatibility) ===

  init: (width: number, height: number) =>
    invoke<boolean>("plugin:cadhy-bridge|viewport_init", { width, height }),

  resize: (width: number, height: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_resize", { width, height }),

  renderFrame: () => invoke<string>("plugin:cadhy-bridge|viewport_render_frame"),

  orbit: (deltaX: number, deltaY: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_orbit", {
      input: { delta_x: deltaX, delta_y: deltaY },
    }),

  pan: (deltaX: number, deltaY: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_pan", {
      input: { delta_x: deltaX, delta_y: deltaY },
    }),

  zoom: (delta: number) => invoke<void>("plugin:cadhy-bridge|viewport_zoom", { input: { delta } }),

  // === SHARED COMMANDS ===

  frameAll: () => invoke<void>("plugin:cadhy-bridge|viewport_frame_all"),

  resetCamera: () => invoke<void>("plugin:cadhy-bridge|viewport_reset_camera"),

  setViewMode: (mode: "solid" | "wireframe") =>
    invoke<void>("plugin:cadhy-bridge|viewport_set_view_mode", { mode }),

  getViewMode: () => invoke<"solid" | "wireframe">("plugin:cadhy-bridge|viewport_get_view_mode"),

  setSettings: (settings: ViewportSettings) =>
    invoke<void>("plugin:cadhy-bridge|viewport_set_settings", { settings }),

  getSettings: () => invoke<ViewportSettings>("plugin:cadhy-bridge|viewport_get_settings"),

  isDirty: () => invoke<boolean>("plugin:cadhy-bridge|viewport_is_dirty"),

  clearDirty: () => invoke<void>("plugin:cadhy-bridge|viewport_clear_dirty"),
}

// Scene API (cadhy-bridge plugin commands)
export interface SceneObjectDto {
  id: string
  name: string
  visible: boolean
  selected: boolean
  object_type: "Mesh" | "Curve" | "Point" | "Light" | "Camera" | "Group"
}

export interface SelectionDto {
  count: number
  ids: string[]
}

export interface TransformDto {
  position: [number, number, number]
  rotation: [number, number, number, number] // quaternion
  scale: [number, number, number]
}

export type PrimitiveParams =
  | { Box: { width: number; height: number; depth: number } }
  | { Sphere: { radius: number } }
  | { Cylinder: { radius: number; height: number } }
  | { Cone: { radius: number; height: number } }
  | { Torus: { major_radius: number; minor_radius: number } }
  | { Plane: { width: number; height: number } }

export const sceneApi = {
  // Object management
  getObjects: () => invoke<SceneObjectDto[]>("plugin:cadhy-bridge|scene_get_objects"),

  addObject: (name: string, transform?: TransformDto) =>
    invoke<string>("plugin:cadhy-bridge|scene_add_object", { name, transform }),

  removeObject: (id: string) =>
    invoke<void>("plugin:cadhy-bridge|scene_remove_object", { id }),

  // Primitives
  addPrimitive: (name: string, primitive: PrimitiveParams, transform?: TransformDto) =>
    invoke<string>("plugin:cadhy-bridge|scene_add_primitive", { name, primitive, transform }),

  addCube: (name?: string, size?: number, transform?: TransformDto) =>
    invoke<string>("plugin:cadhy-bridge|scene_add_cube", { name, size, transform }),

  // Selection
  select: (ids: string[], extend?: boolean) =>
    invoke<SelectionDto>("plugin:cadhy-bridge|scene_select", { ids, extend: extend ?? false }),

  deselectAll: () => invoke<void>("plugin:cadhy-bridge|scene_deselect_all"),

  getSelection: () => invoke<SelectionDto>("plugin:cadhy-bridge|scene_get_selection"),

  // Transforms
  setTransform: (id: string, transform: TransformDto) =>
    invoke<void>("plugin:cadhy-bridge|scene_set_transform", { id, transform }),

  getTransform: (id: string) =>
    invoke<TransformDto>("plugin:cadhy-bridge|scene_get_transform", { id }),
}

// Project API
export const projectApi = {
  newProject: () => invoke<void>("plugin:cadhy-bridge|project_new"),

  undo: () => invoke<void>("plugin:cadhy-bridge|project_undo"),

  redo: () => invoke<void>("plugin:cadhy-bridge|project_redo"),

  canUndo: () => invoke<boolean>("plugin:cadhy-bridge|project_can_undo"),

  canRedo: () => invoke<boolean>("plugin:cadhy-bridge|project_can_redo"),
}

// ============================================================================
// OVERLAY MODE API (Inverted Architecture)
// ============================================================================

export interface ViewportBounds {
  x: number
  y: number
  width: number
  height: number
}

export interface OverlayLayout {
  titlebar_height?: number
  left_panel_width?: number
  right_panel_width?: number
  statusbar_height?: number
  left_collapsed?: boolean
  right_collapsed?: boolean
}

/**
 * Overlay Mode API for the "inverted" architecture where:
 * - Main window = wgpu surface (receives native mouse events)
 * - Child webviews = UI regions only (titlebar, panels, status bar)
 *
 * This provides 100+ FPS rendering with native viewport interaction.
 */
export const overlayApi = {
  /** Initialize overlay mode viewport on the main window */
  init: (width: number, height: number) =>
    invoke<boolean>("plugin:cadhy-bridge|overlay_init", { width, height }),

  /** Check if overlay mode is active */
  isActive: () => invoke<boolean>("plugin:cadhy-bridge|overlay_is_active"),

  /** Get current viewport bounds (area not covered by UI overlays) */
  getViewportBounds: () =>
    invoke<ViewportBounds>("plugin:cadhy-bridge|overlay_get_viewport_bounds"),

  /** Update layout configuration */
  setLayout: (layout: OverlayLayout) =>
    invoke<ViewportBounds>("plugin:cadhy-bridge|overlay_set_layout", { ...layout }),

  /** Toggle left panel visibility */
  toggleLeftPanel: () =>
    invoke<ViewportBounds>("plugin:cadhy-bridge|overlay_toggle_left_panel"),

  /** Toggle right panel visibility */
  toggleRightPanel: () =>
    invoke<ViewportBounds>("plugin:cadhy-bridge|overlay_toggle_right_panel"),

  /** Handle window resize - call when window size changes */
  onResize: (width: number, height: number) =>
    invoke<ViewportBounds>("plugin:cadhy-bridge|overlay_on_resize", { width, height }),

  // Camera controls
  cameraOrbit: (deltaX: number, deltaY: number) =>
    invoke<void>("plugin:cadhy-bridge|overlay_camera_orbit", { delta_x: deltaX, delta_y: deltaY }),

  cameraPan: (deltaX: number, deltaY: number) =>
    invoke<void>("plugin:cadhy-bridge|overlay_camera_pan", { delta_x: deltaX, delta_y: deltaY }),

  cameraZoom: (delta: number) =>
    invoke<void>("plugin:cadhy-bridge|overlay_camera_zoom", { delta }),

  cameraReset: () => invoke<void>("plugin:cadhy-bridge|overlay_camera_reset"),
}

// Helper to detect which UI region this webview represents (in overlay mode)
export type UiRegion = "titlebar" | "left-panel" | "right-panel" | "statusbar" | "main"

export function getCurrentUiRegion(): UiRegion {
  const params = new URLSearchParams(window.location.search)
  const region = params.get("region")
  if (region === "titlebar" || region === "left-panel" || region === "right-panel" || region === "statusbar") {
    return region
  }
  return "main"
}

// ============================================================================
// WGPU VIEWPORT API (Inverted Architecture - lib.rs commands)
// ============================================================================

/**
 * Direct Viewport API for the "inverted" wgpu architecture.
 *
 * These commands talk directly to the desktop app's lib.rs, which controls:
 * - Raw Window with wgpu surface (renders 3D content)
 * - Child WebView for React UI (transparent overlay)
 *
 * The wgpu render thread runs continuously at 100+ FPS in Rust.
 * These commands send messages to that thread via crossbeam channels.
 */
export const wgpuOverlayApi = {
  /** Orbit camera around target */
  orbit: (deltaX: number, deltaY: number) =>
    invoke<void>("viewport_orbit", { delta_x: deltaX, delta_y: deltaY }),

  /** Pan camera (shift + drag) */
  pan: (deltaX: number, deltaY: number) =>
    invoke<void>("viewport_pan", { delta_x: deltaX, delta_y: deltaY }),

  /** Zoom camera (scroll wheel) */
  zoom: (delta: number) => invoke<void>("viewport_zoom", { delta }),

  /** Reset camera to default position */
  resetCamera: () => invoke<void>("viewport_reset_camera"),

  /** Set view mode (solid/wireframe) */
  setViewMode: (mode: "solid" | "wireframe") =>
    invoke<void>("viewport_set_view_mode", { mode }),

  /** Get real-time FPS from wgpu render thread */
  getFps: () => invoke<{ fps: number; frame_time_ms: number }>("viewport_get_fps"),
}

// Update sceneApi to use the new direct command
export const sceneApiDirect = {
  /** Add a cube to the scene */
  addCube: (name: string, size: number) =>
    invoke<string>("scene_add_cube", { name, size }),
}
