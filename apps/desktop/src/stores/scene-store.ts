import { create } from "zustand"
import { sceneApi, type SceneObjectDto, type SelectionDto, type TransformDto } from "@/lib/tauri"

interface SceneState {
  objects: SceneObjectDto[]
  selection: SelectionDto
  isLoading: boolean
  error: string | null
}

interface SceneActions {
  // Data fetching
  fetchObjects: () => Promise<void>
  fetchSelection: () => Promise<void>
  refresh: () => Promise<void>

  // Object management
  addCube: (name?: string, size?: number) => Promise<string | null>
  addSphere: (name?: string, radius?: number) => Promise<string | null>
  addCylinder: (name?: string, radius?: number, height?: number) => Promise<string | null>
  removeObject: (id: string) => Promise<void>

  // Selection
  select: (ids: string[], extend?: boolean) => Promise<void>
  deselectAll: () => Promise<void>
  toggleSelection: (id: string) => Promise<void>
  selectedObjects: () => SceneObjectDto[]

  // Transforms
  getTransform: (id: string) => Promise<TransformDto | null>
  setTransform: (id: string, transform: TransformDto) => Promise<void>
  translateSelection: (dx: number, dy: number, dz: number) => Promise<void>

  // Polling
  startPolling: () => () => void
}

export const useSceneStore = create<SceneState & SceneActions>((set, get) => ({
  objects: [],
  selection: { count: 0, ids: [] },
  isLoading: false,
  error: null,

  fetchObjects: async () => {
    try {
      const objects = await sceneApi.getObjects()
      set({ objects, error: null })
    } catch (e) {
      console.error("Failed to fetch objects:", e)
      set({ error: String(e) })
    }
  },

  fetchSelection: async () => {
    try {
      const selection = await sceneApi.getSelection()
      set({ selection })
    } catch (e) {
      console.error("Failed to fetch selection:", e)
    }
  },

  refresh: async () => {
    set({ isLoading: true })
    await Promise.all([get().fetchObjects(), get().fetchSelection()])
    set({ isLoading: false })
  },

  addCube: async (name, size) => {
    try {
      const cubeName = name || `Cube ${get().objects.length + 1}`
      const id = await sceneApi.addCube(cubeName, size ?? 1.0)
      await get().fetchObjects()
      return id
    } catch (e) {
      console.error("Failed to add cube:", e)
      set({ error: String(e) })
      return null
    }
  },

  addSphere: async (name, radius) => {
    try {
      const sphereName = name || `Sphere ${get().objects.length + 1}`
      const id = await sceneApi.addPrimitive(sphereName, { Sphere: { radius: radius ?? 0.5 } })
      await get().fetchObjects()
      return id
    } catch (e) {
      console.error("Failed to add sphere:", e)
      set({ error: String(e) })
      return null
    }
  },

  addCylinder: async (name, radius, height) => {
    try {
      const cylinderName = name || `Cylinder ${get().objects.length + 1}`
      const id = await sceneApi.addPrimitive(cylinderName, {
        Cylinder: { radius: radius ?? 0.5, height: height ?? 1.0 },
      })
      await get().fetchObjects()
      return id
    } catch (e) {
      console.error("Failed to add cylinder:", e)
      set({ error: String(e) })
      return null
    }
  },

  removeObject: async (id) => {
    try {
      await sceneApi.removeObject(id)
      await get().fetchObjects()
    } catch (e) {
      console.error("Failed to remove object:", e)
      set({ error: String(e) })
    }
  },

  select: async (ids, extend) => {
    try {
      const selection = await sceneApi.select(ids, extend)
      set({ selection })
      await get().fetchObjects() // Refresh to get updated selected states
    } catch (e) {
      console.error("Failed to select:", e)
    }
  },

  deselectAll: async () => {
    try {
      await sceneApi.deselectAll()
      set({ selection: { count: 0, ids: [] } })
      await get().fetchObjects()
    } catch (e) {
      console.error("Failed to deselect:", e)
    }
  },

  toggleSelection: async (id) => {
    const { selection } = get()
    const isSelected = selection.ids.includes(id)
    if (isSelected) {
      // Remove from selection
      const newIds = selection.ids.filter((i) => i !== id)
      if (newIds.length === 0) {
        await get().deselectAll()
      } else {
        await get().select(newIds, false)
      }
    } else {
      // Add to selection (extend)
      await get().select([id], true)
    }
  },

  selectedObjects: () => {
    const { objects, selection } = get()
    return objects.filter((obj) => selection.ids.includes(obj.id))
  },

  getTransform: async (id) => {
    try {
      return await sceneApi.getTransform(id)
    } catch (e) {
      console.error("Failed to get transform:", e)
      return null
    }
  },

  setTransform: async (id, transform) => {
    try {
      await sceneApi.setTransform(id, transform)
    } catch (e) {
      console.error("Failed to set transform:", e)
    }
  },

  translateSelection: async (dx, dy, dz) => {
    const { selection } = get()
    if (selection.ids.length === 0) return

    try {
      // Update each selected object's transform
      for (const id of selection.ids) {
        const transform = await sceneApi.getTransform(id)
        if (transform) {
          // Add delta to current position
          const newTransform: TransformDto = {
            ...transform,
            position: [
              transform.position[0] + dx,
              transform.position[1] + dy,
              transform.position[2] + dz,
            ],
          }
          await sceneApi.setTransform(id, newTransform)
        }
      }
    } catch (e) {
      console.error("Failed to translate selection:", e)
    }
  },

  // Start polling for scene updates (returns cleanup function)
  startPolling: () => {
    const fetchData = () => {
      get().fetchObjects()
      get().fetchSelection()
    }

    // Initial fetch
    fetchData()

    // Poll every 1 second
    const intervalId = setInterval(fetchData, 1000)

    return () => clearInterval(intervalId)
  },
}))
