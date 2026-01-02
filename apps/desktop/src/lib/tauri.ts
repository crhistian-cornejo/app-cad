import { invoke } from "@tauri-apps/api/core"

// Viewport API
export const viewportApi = {
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

  frameAll: () =>
    invoke<void>("plugin:cadhy-bridge|viewport_frame_all"),

  resetCamera: () =>
    invoke<void>("plugin:cadhy-bridge|viewport_reset_camera"),
}

// Scene API
export const sceneApi = {
  addCube: (name: string, size: number) =>
    invoke<string>("plugin:cadhy-bridge|scene_add_cube", { name, size }),
}
