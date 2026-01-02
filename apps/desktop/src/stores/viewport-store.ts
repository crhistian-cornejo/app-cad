import { create } from "zustand"
import { viewportApi, wgpuOverlayApi } from "@/lib/tauri"

type ViewMode = "solid" | "wireframe"
type Projection = "perspective" | "orthographic"

interface ViewportState {
  width: number
  height: number
  viewMode: ViewMode
  projection: Projection
  isInitialized: boolean
  cursorPosition: { x: number; y: number; z: number }
  fps: number
  frameTime: number
  objectCount: number
}

interface ViewportActions {
  setSize: (width: number, height: number) => void
  setViewMode: (mode: ViewMode) => void
  setProjection: (projection: Projection) => void
  setInitialized: (initialized: boolean) => void
  setCursorPosition: (x: number, y: number, z: number) => void
  setFps: (fps: number, frameTime: number) => void
  setObjectCount: (count: number) => void
  frameAll: () => void
  resetCamera: () => void
  setView: (view: "front" | "back" | "right" | "left" | "top" | "bottom") => void
  setCamera: (position: [number, number, number], target: [number, number, number]) => void
  startFpsPolling: () => () => void
}

export const useViewportStore = create<ViewportState & ViewportActions>((set, get) => ({
  width: 800,
  height: 600,
  viewMode: "solid",
  projection: "perspective",
  isInitialized: false,
  cursorPosition: { x: 0, y: 0, z: 0 },
  fps: 0,
  frameTime: 0,
  objectCount: 0,

  setSize: (width, height) => set({ width, height }),
  setViewMode: (viewMode) => set({ viewMode }),
  setProjection: (projection) => set({ projection }),
  setInitialized: (isInitialized) => set({ isInitialized }),
  setCursorPosition: (x, y, z) => set({ cursorPosition: { x, y, z } }),
  setFps: (fps, frameTime) => set({ fps, frameTime }),
  setObjectCount: (count) => set({ objectCount: count }),

  frameAll: async () => {
    try {
      await viewportApi.frameAll()
    } catch (e) {
      console.error("Frame all failed:", e)
    }
  },

  resetCamera: async () => {
    try {
      await wgpuOverlayApi.resetCamera()
    } catch (e) {
      console.error("Reset camera failed:", e)
    }
  },

  setView: async (view) => {
    // Camera positions for standard orthographic views
    const viewPositions: Record<
      string,
      { position: [number, number, number]; target: [number, number, number] }
    > = {
      front: { position: [0, 0, 10], target: [0, 0, 0] },
      back: { position: [0, 0, -10], target: [0, 0, 0] },
      right: { position: [10, 0, 0], target: [0, 0, 0] },
      left: { position: [-10, 0, 0], target: [0, 0, 0] },
      top: { position: [0, 10, 0], target: [0, 0, 0] },
      bottom: { position: [0, -10, 0], target: [0, 0, 0] },
    }

    const viewConfig = viewPositions[view]
    if (viewConfig) {
      try {
        await wgpuOverlayApi.setCamera(viewConfig.position, viewConfig.target)
      } catch (e) {
        console.error("Failed to set view:", e)
      }
    }
  },

  setCamera: async (position, target) => {
    try {
      await wgpuOverlayApi.setCamera(position, target)
    } catch (e) {
      console.error("Failed to set camera:", e)
    }
  },

  // Start polling FPS from wgpu backend
  startFpsPolling: () => {
    const fetchFps = async () => {
      try {
        const { fps, frame_time_ms } = await wgpuOverlayApi.getFps()
        set({ fps, frameTime: frame_time_ms })
      } catch (e) {
        // Silently ignore - wgpu might not be ready yet
      }
    }

    // Poll every 500ms for smooth updates
    const intervalId = setInterval(fetchFps, 500)
    fetchFps() // Initial fetch

    // Return cleanup function
    return () => clearInterval(intervalId)
  },
}))
