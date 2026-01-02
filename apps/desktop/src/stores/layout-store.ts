import { create } from "zustand"
import { persist } from "zustand/middleware"

interface LayoutState {
  panels: {
    left: boolean
    right: boolean
  }
  theme: "light" | "dark" | "system"
  // Appearance settings
  borderRadius: number // 0 to 16
  accentColor: string
  // Viewport settings
  showGrid: boolean
  showAxes: boolean
  antiAliasing: boolean
}

interface LayoutActions {
  togglePanel: (panel: "left" | "right") => void
  setTheme: (theme: "light" | "dark" | "system") => void
  setBorderRadius: (radius: number) => void
  setAccentColor: (color: string) => void
  setShowGrid: (show: boolean) => void
  setShowAxes: (show: boolean) => void
  setAntiAliasing: (enabled: boolean) => void
}

export const useLayoutStore = create<LayoutState & LayoutActions>()(
  persist(
    (set) => ({
      panels: {
        left: true,
        right: true,
      },
      theme: "dark",
      // Default appearance settings
      borderRadius: 0,
      accentColor: "#3b82f6", // blue-500
      // Default viewport settings
      showGrid: true,
      showAxes: true,
      antiAliasing: true,

      togglePanel: (panel) =>
        set((state) => ({
          panels: { ...state.panels, [panel]: !state.panels[panel] },
        })),
      setTheme: (theme) => set({ theme }),
      setBorderRadius: (borderRadius) => set({ borderRadius }),
      setAccentColor: (accentColor) => set({ accentColor }),
      setShowGrid: (showGrid) => set({ showGrid }),
      setShowAxes: (showAxes) => set({ showAxes }),
      setAntiAliasing: (antiAliasing) => set({ antiAliasing }),
    }),
    { name: "cadhy-layout" }
  )
)
