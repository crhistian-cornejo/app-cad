/**
 * ViewportPanel - CADHY 3D Viewport with wgpu Rendering
 *
 * Uses cadhy-bridge for wgpu rendering with optimized render loop.
 */

import { useCallback, useEffect, useRef, useState, type PointerEvent, type WheelEvent } from "react"
import {
  Add01Icon,
  Camera01Icon,
  CubeIcon,
  GridIcon,
  MaximizeScreenIcon,
  Search01Icon,
} from "@hugeicons/core-free-icons"
import { HugeiconsIcon } from "@hugeicons/react"
import { Button, Kbd, Tooltip, TooltipContent, TooltipTrigger } from "@cadhy/ui"
import { CommandPalette } from "@/components/pallete/CommandPalette"
import { useViewportStore } from "@/stores/viewport-store"
import { viewportApi, sceneApi } from "@/lib/tauri"

// ============================================================================
// VIEWPORT TOOLBAR
// ============================================================================

function ViewportToolbar() {
  const { viewMode, setViewMode, frameAll, resetCamera } = useViewportStore()
  const [isAddingCube, setIsAddingCube] = useState(false)

  const handleAddCube = async () => {
    if (isAddingCube) return
    setIsAddingCube(true)
    try {
      await sceneApi.addCube("Cube", 1.0)
    } catch (e) {
      console.error("Failed to add cube:", e)
    } finally {
      setIsAddingCube(false)
    }
  }

  const handleSetViewMode = async (mode: "solid" | "wireframe") => {
    try {
      await viewportApi.setViewMode(mode)
      setViewMode(mode)
    } catch (e) {
      console.error("Failed to set view mode:", e)
    }
  }

  const handleResetCamera = async () => {
    try {
      await viewportApi.resetCamera()
      resetCamera()
    } catch (e) {
      console.error("Failed to reset camera:", e)
    }
  }

  return (
    <div className="absolute right-2 top-2 flex flex-col gap-1 rounded-lg bg-card/90 p-1.5 backdrop-blur-md border border-border/20 shadow-lg z-10">
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="size-8"
            onClick={handleAddCube}
            disabled={isAddingCube}
          >
            <HugeiconsIcon icon={Add01Icon} size={18} />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Add Cube</TooltipContent>
      </Tooltip>

      <div className="my-1 h-px bg-border/50" />

      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant={viewMode === "solid" ? "secondary" : "ghost"}
            size="icon"
            className="size-8"
            onClick={() => handleSetViewMode("solid")}
          >
            <HugeiconsIcon icon={CubeIcon} size={18} />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Solid View (Z)</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant={viewMode === "wireframe" ? "secondary" : "ghost"}
            size="icon"
            className="size-8"
            onClick={() => handleSetViewMode("wireframe")}
          >
            <HugeiconsIcon icon={GridIcon} size={18} />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Wireframe View (Z)</TooltipContent>
      </Tooltip>

      <div className="my-1 h-px bg-border/50" />

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="icon" className="size-8" onClick={frameAll}>
            <HugeiconsIcon icon={MaximizeScreenIcon} size={18} />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Frame All (Home)</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="icon" className="size-8" onClick={handleResetCamera}>
            <HugeiconsIcon icon={Camera01Icon} size={18} />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Reset Camera</TooltipContent>
      </Tooltip>
    </div>
  )
}

// ============================================================================
// VIEWPORT CANVAS with optimized render loop
// ============================================================================

// Note: wgpu window fills the entire container. Toolbars overlay on top with z-index.

function ViewportCanvas() {
  const containerRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const animationFrameRef = useRef<number | undefined>(undefined)
  const isDragging = useRef(false)
  const dragMode = useRef<"orbit" | "pan">("orbit")
  const lastMousePos = useRef({ x: 0, y: 0 })

  const { isInitialized, setInitialized, frameAll, resetCamera } = useViewportStore()

  const [size, setSize] = useState({ width: 0, height: 0 })

  // State to track rendering mode
  const [nativeMode, setNativeMode] = useState(false)

  // Initialize viewport
  useEffect(() => {
    let mounted = true

    const initializeViewport = async () => {
      if (!containerRef.current) return

      try {
        const rect = containerRef.current.getBoundingClientRect()
        const width = Math.floor(rect.width)
        const height = Math.floor(rect.height)
        const x = Math.floor(rect.left)
        const y = Math.floor(rect.top)

        console.log("[Viewport] Initializing with bounds:", { width, height, x, y })

        // Try native viewport first
        try {
          await viewportApi.initNative(width, height, x, y)
          if (mounted) {
            setInitialized(true)
            setNativeMode(true)
            console.log("[Viewport] Native viewport initialized successfully")
          }
          return
        } catch (nativeError) {
          console.log("[Viewport] Native viewport failed, falling back to offscreen:", nativeError)
        }

        // Fallback to offscreen
        await viewportApi.init(width, height)
        if (mounted) {
          setInitialized(true)
          setNativeMode(false)
          console.log("[Viewport] Offscreen viewport initialized successfully")
        }
      } catch (error) {
        console.error("[Viewport] Initialization failed:", error)
      }
    }

    initializeViewport()

    return () => {
      mounted = false
    }
  }, [setInitialized])

  // Handle container resize
  useEffect(() => {
    if (!containerRef.current || !isInitialized) return

    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (!entry) return

      const { width, height } = entry.contentRect
      setSize({ width: Math.floor(width), height: Math.floor(height) })

      // Update viewport bounds
      const updateBounds = async () => {
        try {
          await viewportApi.resize(Math.floor(width), Math.floor(height))
        } catch (e) {
          console.error("[Viewport] Resize error:", e)
        }
      }

      updateBounds()
    })

    resizeObserver.observe(containerRef.current)
    return () => resizeObserver.disconnect()
  }, [isInitialized])

  // In native mode, listen for window move events to reposition viewport
  useEffect(() => {
    if (!nativeMode || !containerRef.current) return

    // Function to update viewport bounds - wgpu fills entire container
    const updateViewportBounds = async () => {
      if (!containerRef.current) return
      const rect = containerRef.current.getBoundingClientRect()
      const x = Math.floor(rect.left)
      const y = Math.floor(rect.top)
      const w = Math.floor(rect.width)
      const h = Math.floor(rect.height)

      console.log("[Viewport] Container bounds:", { x, y, w, h })

      try {
        await viewportApi.updateBounds(x, y, w, h)
      } catch (e) {
        console.error("[Viewport] Update bounds error:", e)
      }
    }

    // Update bounds on mount and when window moves
    updateViewportBounds()

    const handleWindowMove = () => {
      updateViewportBounds()
    }

    // Listen for window move events (this is a simplified approach)
    const interval = setInterval(handleWindowMove, 100)

    return () => clearInterval(interval)
  }, [nativeMode])

  // Camera control handlers
  const handlePointerDown = useCallback((e: PointerEvent<HTMLDivElement>) => {
    // Alt+Click or Middle mouse button for camera control
    if (e.button === 1 || (e.button === 0 && e.altKey)) {
      e.preventDefault()
      isDragging.current = true
      lastMousePos.current = { x: e.clientX, y: e.clientY }
      dragMode.current = e.shiftKey ? "pan" : "orbit"
      console.log(
        `[Viewport] Drag started: mode=${dragMode.current}, pos=(${e.clientX}, ${e.clientY})`,
      )
      ;(e.target as HTMLElement).setPointerCapture(e.pointerId)
    }
  }, [])

  const handlePointerMove = useCallback(
    (e: PointerEvent<HTMLDivElement>) => {
      if (!isDragging.current || !isInitialized) return

      const deltaX = e.clientX - lastMousePos.current.x
      const deltaY = e.clientY - lastMousePos.current.y

      try {
        if (dragMode.current === "orbit") {
          viewportApi.orbitNative(deltaX * 0.01, deltaY * 0.01)
        } else {
          viewportApi.panNative(deltaX * 0.01, deltaY * 0.01)
        }
      } catch (error) {
        console.error("[Viewport] Camera control error:", error)
      }

      lastMousePos.current = { x: e.clientX, y: e.clientY }
    },
    [isInitialized],
  )

  const handlePointerUp = useCallback((e: PointerEvent<HTMLDivElement>) => {
    if (isDragging.current) {
      isDragging.current = false
      ;(e.target as HTMLElement).releasePointerCapture(e.pointerId)
      console.log("[Viewport] Drag ended")
    }
  }, [])

  const handleWheel = useCallback(
    (e: WheelEvent<HTMLDivElement>) => {
      if (!isInitialized) return
      e.preventDefault()

      try {
        viewportApi.zoomNative(e.deltaY * -0.001)
      } catch (error) {
        console.error("[Viewport] Zoom error:", error)
      }
    },
    [isInitialized],
  )

  // Render loop for offscreen mode
  useEffect(() => {
    if (!isInitialized || nativeMode) return

    const render = () => {
      try {
        viewportApi.renderFrame()
      } catch (error) {
        console.error("[Viewport] Render error:", error)
      }
      animationFrameRef.current = requestAnimationFrame(render)
    }

    render()

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current)
      }
    }
  }, [isInitialized, nativeMode])

  return (
    <div
      ref={containerRef}
      role="application"
      aria-label="3D Viewport"
      className={`absolute inset-0 cursor-crosshair touch-none ${nativeMode ? "bg-transparent" : "bg-[#1a1a1a]"}`}
      style={nativeMode ? { backgroundColor: "transparent" } : undefined}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onWheel={handleWheel}
    >
      {/* Loading state */}
      {!isInitialized && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center pointer-events-none">
            <div className="mb-4 opacity-10">
              <HugeiconsIcon icon={CubeIcon} size={80} />
            </div>
            <p className="text-sm text-muted-foreground/60">Initializing wgpu...</p>
          </div>
        </div>
      )}

      {/* Hidden in native mode to allow transparent background */}
      {!nativeMode && (
        <div className="absolute inset-0 pointer-events-none z-20">
          {/* Top Left */}
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            className="absolute top-0 left-0 fill-card"
            aria-hidden="true"
          >
            <path d="M0 0 L16 0 Q0 0 0 16 L0 0 Z" />
            <path
              d="M16 0 Q0 0 0 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="0.5"
              className="text-border/30"
            />
          </svg>
          {/* Top Right */}
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            className="absolute top-0 right-0 fill-card"
            aria-hidden="true"
          >
            <path d="M16 0 L0 0 Q16 0 16 16 L16 0 Z" />
            <path
              d="M0 0 Q16 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="0.5"
              className="text-border/30"
            />
          </svg>
          {/* Bottom Left */}
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            className="absolute bottom-0 left-0 fill-card"
            aria-hidden="true"
          >
            <path d="M0 16 L16 16 Q0 16 0 0 L0 16 Z" />
            <path
              d="M16 16 Q0 16 0 0"
              fill="none"
              stroke="currentColor"
              strokeWidth="0.5"
              className="text-border/30"
            />
          </svg>
          {/* Bottom Right */}
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            className="absolute bottom-0 right-0 fill-card"
            aria-hidden="true"
          >
            <path d="M16 16 L0 16 Q16 16 16 0 L16 16 Z" />
            <path
              d="M0 16 Q16 16 16 0"
              fill="none"
              stroke="currentColor"
              strokeWidth="0.5"
              className="text-border/30"
            />
          </svg>
        </div>
      )}

      {/* Canvas is only used for offscreen mode - hidden in native mode */}
      <canvas
        ref={canvasRef}
        className={`absolute inset-0 w-full h-full ${isInitialized && !nativeMode ? "" : "opacity-0"}`}
        style={{ imageRendering: "auto", pointerEvents: nativeMode ? "none" : "auto" }}
      />
    </div>
  )
}

// ============================================================================
// BOTTOM TOOLBAR - Horizontal controls bar
// ============================================================================

function BottomToolbar({ onOpenCommandPalette }: { onOpenCommandPalette: () => void }) {
  const [lightIntensity, setLightIntensity] = useState(64)
  const [lightAngle, setLightAngle] = useState(0)

  return (
    <div className="absolute bottom-0 left-0 right-0 flex items-center justify-center pointer-events-none z-10">
      <div className="flex items-end pointer-events-auto">
        {/* Left corner decoration */}
        <svg
          width="10"
          height="36"
          viewBox="0 0 10 10"
          className="block shrink-0 fill-card"
          preserveAspectRatio="none"
          aria-hidden="true"
        >
          <path d="M10 0 L10 10 L0 10 Q10 10 10 0" />
          <path
            d="M10 0 Q10 10 0 10"
            fill="none"
            stroke="currentColor"
            strokeWidth="0.5"
            className="text-border/20"
          />
        </svg>

        {/* Main toolbar content */}
        <div className="flex items-center h-9 px-3 bg-card border-t border-border/20 gap-3 rounded-t-[var(--radius)]">
          {/* Command Palette button */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant="ghost" size="icon" className="size-8" onClick={onOpenCommandPalette}>
                <HugeiconsIcon icon={Search01Icon} size={18} />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="top">
              <div className="flex items-center gap-2">
                <span>Command Palette</span>
                <Kbd variant="inverted">⌘K</Kbd>
              </div>
            </TooltipContent>
          </Tooltip>

          {/* Separator */}
          <div className="w-px h-9 bg-border/30" />

          {/* Light Intensity slider */}
          <div className="flex items-center gap-2">
            <span className="text-[10px] text-muted-foreground uppercase tracking-wider whitespace-nowrap">
              Light Intensity
            </span>
            <div className="relative w-20 flex items-center">
              <input
                type="range"
                min="0"
                max="100"
                step="1"
                value={lightIntensity}
                onChange={(e) => setLightIntensity(Number(e.target.value))}
                className="w-full h-1 bg-muted rounded-full appearance-none cursor-pointer accent-primary"
              />
            </div>
            <span className="text-[10px] text-muted-foreground font-medium w-6 text-right">
              {lightIntensity}
            </span>
          </div>

          {/* Separator */}
          <div className="w-px h-9 bg-border/30" />

          {/* Light Angle slider */}
          <div className="flex items-center gap-2">
            <span className="text-[10px] text-muted-foreground uppercase tracking-wider whitespace-nowrap">
              Light Angle
            </span>
            <div className="relative w-20 flex items-center">
              <input
                type="range"
                min="-180"
                max="180"
                step="5"
                value={lightAngle}
                onChange={(e) => setLightAngle(Number(e.target.value))}
                className="w-full h-1 bg-muted rounded-full appearance-none cursor-pointer accent-primary"
              />
            </div>
            <span className="text-[10px] text-muted-foreground font-medium w-6 text-right">
              {lightAngle}°
            </span>
          </div>
        </div>

        {/* Right corner decoration */}
        <svg
          width="10"
          height="36"
          viewBox="0 0 10 10"
          className="block shrink-0 fill-card"
          preserveAspectRatio="none"
          aria-hidden="true"
        >
          <path d="M0 0 L10 0 Q10 10 0 10 L0 0 Z" />
          <path
            d="M0 0 Q10 10 0 10"
            fill="none"
            stroke="currentColor"
            strokeWidth="0.5"
            className="text-border/20"
          />
        </svg>
      </div>
    </div>
  )
}

// ============================================================================
// MAIN VIEWPORT PANEL
// ============================================================================

export function ViewportPanel() {
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)

  // Handle keyboard shortcut for command palette
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+K or Ctrl+K to open command palette
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault()
        setCommandPaletteOpen(true)
      }
      // Escape to close
      if (e.key === "Escape") {
        setCommandPaletteOpen(false)
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [])

  return (
    <div className="relative h-full w-full overflow-hidden">
      <ViewportCanvas />
      <ViewportToolbar />
      <BottomToolbar onOpenCommandPalette={() => setCommandPaletteOpen(true)} />
      <CommandPalette open={commandPaletteOpen} onOpenChange={setCommandPaletteOpen} />
    </div>
  )
}
