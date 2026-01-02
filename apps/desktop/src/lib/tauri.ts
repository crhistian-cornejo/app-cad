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

/** Pick result from viewport - can be background, gizmo axis, or object */
export type PickResult =
  | { type: "none" }
  | { type: "gizmo"; axis: "x" | "y" | "z" | "xy" | "xz" | "yz" | "all" | "none" }
  | { type: "object"; id: string }

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
  /** Orbit camera around target - sends to WgpuOverlay render thread */
  orbit: (deltaX: number, deltaY: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_overlay_orbit", { delta_x: deltaX, delta_y: deltaY }),

  /** Pan camera (shift + drag) - sends to WgpuOverlay render thread */
  pan: (deltaX: number, deltaY: number) =>
    invoke<void>("plugin:cadhy-bridge|viewport_overlay_pan", { delta_x: deltaX, delta_y: deltaY }),

  /** Zoom camera (scroll wheel) - sends to WgpuOverlay render thread */
  zoom: (delta: number) => invoke<void>("plugin:cadhy-bridge|viewport_overlay_zoom", { delta }),

  /** Reset camera to default position */
  resetCamera: () => invoke<void>("plugin:cadhy-bridge|viewport_overlay_reset_camera"),

  /** Set view mode (solid/wireframe) */
  setViewMode: (mode: "solid" | "wireframe") =>
    invoke<void>("plugin:cadhy-bridge|viewport_overlay_set_view_mode", { mode }),

  /** Get real-time FPS from wgpu render thread */
  getFps: () => invoke<{ fps: number; frame_time_ms: number }>("plugin:cadhy-bridge|viewport_get_fps"),

  /** Set camera position and target for view presets */
  setCamera: (position: [number, number, number], target: [number, number, number]) =>
    invoke<void>("plugin:cadhy-bridge|viewport_overlay_set_camera", {
      position,
      target,
    }),

  /** Pick object or gizmo at screen coordinates */
  pick: (x: number, y: number) =>
    invoke<PickResult>("plugin:cadhy-bridge|viewport_overlay_pick", { x, y }),

  /** Set gizmo mode (translate, rotate, scale) */
  setGizmoMode: (mode: "translate" | "rotate" | "scale") =>
    invoke<void>("plugin:cadhy-bridge|viewport_overlay_set_gizmo_mode", { mode }),
}

// Update sceneApi to use the new direct command
export const sceneApiDirect = {
  /** Add a cube to the scene */
  addCube: (name: string, size: number) =>
    invoke<string>("scene_add_cube", { name, size }),
}

// ============================================================================
// CAD API (B-Rep Operations via OpenCASCADE)
// ============================================================================

/** Boolean operation types */
export type BooleanOp = "fuse" | "cut" | "common"

/** Result from CAD operations */
export interface CadOperationResult {
  id: string
  name: string
  success: boolean
  message?: string
}

/**
 * CAD API for B-Rep solid modeling operations.
 *
 * These operations work on OpenCASCADE B-Rep shapes stored in the backend.
 * Objects created via `cadApi.createPrimitive()` have associated B-Rep data
 * that enables boolean operations, fillets, chamfers, etc.
 */
export const cadApi = {
  // === PRIMITIVE CREATION (with B-Rep storage) ===

  /**
   * Create a CAD primitive with B-Rep data storage.
   * Unlike sceneApi.addPrimitive, this stores the full B-Rep shape
   * enabling boolean operations and other CAD operations.
   */
  createPrimitive: (name: string, primitive: PrimitiveParams, transform?: TransformDto) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_create_primitive", {
      name,
      primitive,
      transform,
    }),

  // === BOOLEAN OPERATIONS ===

  /**
   * Generic boolean operation on two objects.
   * @param objectA - First object UUID
   * @param objectB - Second object UUID
   * @param operation - "fuse" (union), "cut" (subtract B from A), or "common" (intersection)
   * @param resultName - Optional name for the resulting object
   * @param keepOriginals - If true, keeps original objects; if false (default), removes them
   */
  boolean: (
    objectA: string,
    objectB: string,
    operation: BooleanOp,
    resultName?: string,
    keepOriginals?: boolean
  ) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_boolean", {
      object_a: objectA,
      object_b: objectB,
      operation,
      result_name: resultName,
      keep_originals: keepOriginals,
    }),

  /**
   * Fuse (union) two objects into one.
   * Creates a new shape that is the combination of both objects.
   */
  fuse: (objectA: string, objectB: string, resultName?: string, keepOriginals?: boolean) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_fuse", {
      object_a: objectA,
      object_b: objectB,
      result_name: resultName,
      keep_originals: keepOriginals,
    }),

  /**
   * Cut (subtract) objectB from objectA.
   * Creates a new shape with objectB removed from objectA.
   */
  cut: (objectA: string, objectB: string, resultName?: string, keepOriginals?: boolean) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_cut", {
      object_a: objectA,
      object_b: objectB,
      result_name: resultName,
      keep_originals: keepOriginals,
    }),

  /**
   * Common (intersection) of two objects.
   * Creates a new shape that is only the shared volume.
   */
  common: (objectA: string, objectB: string, resultName?: string, keepOriginals?: boolean) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_common", {
      object_a: objectA,
      object_b: objectB,
      result_name: resultName,
      keep_originals: keepOriginals,
    }),

  // === FILLET & CHAMFER ===

  /**
   * Apply fillet (rounded edges) to all edges of an object.
   * @param objectId - Object UUID
   * @param radius - Fillet radius
   */
  fillet: (objectId: string, radius: number) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_fillet", {
      object_id: objectId,
      radius,
    }),

  /**
   * Apply chamfer (beveled edges) to all edges of an object.
   * @param objectId - Object UUID
   * @param distance - Chamfer distance
   */
  chamfer: (objectId: string, distance: number) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_chamfer", {
      object_id: objectId,
      distance,
    }),

  // === SHELL & OFFSET ===

  /**
   * Create a hollow shell from a solid.
   * @param objectId - Object UUID
   * @param thickness - Wall thickness (positive = inward, negative = outward)
   */
  shell: (objectId: string, thickness: number) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_shell", {
      object_id: objectId,
      thickness,
    }),

  /**
   * Offset a solid inward or outward.
   * @param objectId - Object UUID
   * @param offset - Offset distance (positive = outward, negative = inward)
   */
  offset: (objectId: string, offset: number) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_offset", {
      object_id: objectId,
      offset,
    }),

  // === TRANSFORM OPERATIONS (modifies B-Rep geometry) ===

  /**
   * Translate (move) a shape's geometry.
   * This modifies the actual B-Rep, not just the scene transform.
   */
  translate: (objectId: string, dx: number, dy: number, dz: number) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_translate", {
      object_id: objectId,
      dx,
      dy,
      dz,
    }),

  /**
   * Rotate a shape's geometry around an axis.
   * @param objectId - Object UUID
   * @param origin - Point on the rotation axis [x, y, z]
   * @param axis - Axis direction [x, y, z]
   * @param angleDegrees - Rotation angle in degrees
   */
  rotate: (
    objectId: string,
    origin: [number, number, number],
    axis: [number, number, number],
    angleDegrees: number
  ) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_rotate", {
      object_id: objectId,
      origin,
      axis,
      angle_degrees: angleDegrees,
    }),

  /**
   * Mirror a shape across a plane.
   * @param objectId - Object UUID
   * @param origin - Point on the mirror plane [x, y, z]
   * @param normal - Normal direction of the plane [x, y, z]
   */
  mirror: (
    objectId: string,
    origin: [number, number, number],
    normal: [number, number, number]
  ) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_mirror", {
      object_id: objectId,
      origin,
      normal,
    }),

  /**
   * Scale a shape uniformly from a center point.
   * @param objectId - Object UUID
   * @param center - Scale center point [x, y, z]
   * @param factor - Scale factor (must be positive)
   */
  scale: (objectId: string, center: [number, number, number], factor: number) =>
    invoke<CadOperationResult>("plugin:cadhy-bridge|cad_scale", {
      object_id: objectId,
      center,
      factor,
    }),
}
