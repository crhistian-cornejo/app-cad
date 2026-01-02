import { create } from "zustand"
import { viewportApi } from "@/lib/tauri"

type ViewMode = "solid" | "wireframe"

interface ViewportState {
  width: number
  height: number
  viewMode: ViewMode
  isInitialized: boolean
  cursorPosition: { x: number; y: number; z: number }
  fps: number
  frameTime: number
  objectCount: number
}

interface ViewportActions {
  setSize: (width: number, height: number) => void
  setViewMode: (mode: ViewMode) => void
  setInitialized: (initialized: boolean) => void
  setCursorPosition: (x: number, y: number, z: number) => void
  setFps: (fps: number, frameTime: number) => void
  setObjectCount: (count: number) => void
  frameAll: () => void
  resetCamera: () => void
}

export const useViewportStore = create<ViewportState & ViewportActions>((set) => ({
  width: 800,
  height: 600,
  viewMode: "solid",
  isInitialized: false,
  cursorPosition: { x: 0, y: 0, z: 0 },
  fps: 0,
  frameTime: 0,
  objectCount: 0,

  setSize: (width, height) => set({ width, height }),
  setViewMode: (viewMode) => set({ viewMode }),
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
      await viewportApi.frameAll() // Use frameAll as resetCamera
    } catch (e) {
      console.error("Reset camera failed:", e)
    }
  },
}))
