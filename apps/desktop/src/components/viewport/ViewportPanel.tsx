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
import { Button, Kbd, Slider, Tooltip, TooltipContent, TooltipTrigger } from "@cadhy/ui"
import { useViewportStore } from "@/stores/viewport-store"
import { sceneApiDirect, wgpuOverlayApi } from "@/lib/tauri"

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
      await sceneApiDirect.addCube("Cube", 1.0)
    } catch (e) {
      console.error("Failed to add cube:", e)
    } finally {
      setIsAddingCube(false)
    }
  }

  const handleSetViewMode = async (mode: "solid" | "wireframe") => {
    try {
      await wgpuOverlayApi.setViewMode(mode)
      setViewMode(mode)
    } catch (e) {
      console.error("Failed to set view mode:", e)
    }
  }

  const handleResetCamera = async () => {
    try {
      await wgpuOverlayApi.resetCamera()
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

  // In inverted architecture, wgpu is ALWAYS running (started in lib.rs)
  // No initialization needed - mark as ready immediately
  useEffect(() => {
    setInitialized(true)
    console.log("[Viewport] Inverted architecture: wgpu renders behind, React UI on top")
  }, [setInitialized])

  // Track window size for UI layout (wgpu resize is handled in Rust via WindowEvent)
  useEffect(() => {
    if (!isInitialized) return

    const handleResize = () => {
      setSize({ width: window.innerWidth, height: window.innerHeight })
    }

    handleResize()
    window.addEventListener("resize", handleResize)
    return () => window.removeEventListener("resize", handleResize)
  }, [isInitialized])

  // Camera control handlers
  const handlePointerDown = useCallback((e: PointerEvent<HTMLDivElement>) => {
    // Left click = orbit, Middle click = pan, Shift+Left = pan
    if (e.button === 0 || e.button === 1) {
      e.preventDefault()
      isDragging.current = true
      lastMousePos.current = { x: e.clientX, y: e.clientY }
      // Middle mouse or Shift+Left = pan, otherwise orbit
      dragMode.current = e.button === 1 || e.shiftKey ? "pan" : "orbit"
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
        // Use wgpuOverlayApi for inverted architecture
        // Pass raw pixel deltas - sensitivity is handled in Rust
        if (dragMode.current === "orbit") {
          wgpuOverlayApi.orbit(deltaX, deltaY)
        } else {
          wgpuOverlayApi.pan(deltaX, deltaY)
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
        // Pass raw delta - sensitivity handled in Rust
        wgpuOverlayApi.zoom(-e.deltaY)
      } catch (error) {
        console.error("[Viewport] Zoom error:", error)
      }
    },
    [isInitialized],
  )

  // No render loop needed - wgpu runs continuously in Rust

  return (
    <div
      ref={containerRef}
      role="application"
      aria-label="3D Viewport"
      // Inverted architecture: transparent so wgpu content shows through
      // wgpu renders to entire window BEHIND this webview
      className="absolute inset-0 cursor-crosshair touch-none bg-transparent"
      style={{ backgroundColor: "transparent", pointerEvents: "auto" }}
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

      {/* Corner decorations */}
      <div className="absolute inset-0 pointer-events-none z-20">
        {/* Top Left */}
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          className="absolute -top-[1px] -left-[1px] fill-card"
          aria-hidden="true"
        >
          <path d="M24 0 L0 0 L0 24 L1 24 L1 9 Q1 1 9 1 L24 1 Z" />
          <path
            d="M24 1 L9 1 Q1 1 1 9 L1 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1"
            className="text-border"
          />
        </svg>
        {/* Top Right */}
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          className="absolute -top-[1px] -right-[1px] fill-card"
          aria-hidden="true"
        >
          <path d="M0 0 L24 0 L24 24 L23 24 L23 9 Q23 1 15 1 L0 1 Z" />
          <path
            d="M0 1 L15 1 Q23 1 23 9 L23 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1"
            className="text-border"
          />
        </svg>
        {/* Bottom Left */}
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          className="absolute -bottom-[1px] -left-[1px] fill-card"
          aria-hidden="true"
        >
          <path d="M24 24 L0 24 L0 0 L1 0 L1 15 Q1 23 9 23 L24 23 Z" />
          <path
            d="M24 23 L9 23 Q1 23 1 15 L1 0"
            fill="none"
            stroke="currentColor"
            strokeWidth="1"
            className="text-border"
          />
        </svg>
        {/* Bottom Right */}
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          className="absolute -bottom-[1px] -right-[1px] fill-card"
          aria-hidden="true"
        >
          <path d="M0 24 L24 24 L24 0 L23 0 L23 15 Q23 23 15 23 L0 23 Z" />
          <path
            d="M0 23 L15 23 Q23 23 23 15 L23 0"
            fill="none"
            stroke="currentColor"
            strokeWidth="1"
            className="text-border"
          />
        </svg>
      </div>

      {/* Canvas not needed in inverted architecture - wgpu renders directly */}
      <canvas
        ref={canvasRef}
        className="absolute inset-0 w-full h-full opacity-0 pointer-events-none"
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
            <div className="w-24">
              <Slider
                value={[lightIntensity]}
                min={0}
                max={100}
                step={1}
                onValueChange={(val) => setLightIntensity(val[0])}
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
            <div className="w-24">
              <Slider
                value={[lightAngle]}
                min={-180}
                max={180}
                step={5}
                onValueChange={(val) => setLightAngle(val[0])}
              />
            </div>
            <span className="text-[10px] text-muted-foreground font-medium w-8 text-right">
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
          <path d="M0 0 L0 10 L10 10 Q0 10 0 0" />
          <path
            d="M0 0 Q0 10 10 10"
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
  // const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)

  // Handle keyboard shortcut for command palette - DISABLED
  // useEffect(() => {
  //   const handleKeyDown = (e: KeyboardEvent) => {
  //     // Cmd+K or Ctrl+K to open command palette
  //     if ((e.metaKey || e.ctrlKey) && e.key === "k") {
  //       e.preventDefault()
  //       setCommandPaletteOpen(true)
  //     }
  //     // Escape to close
  //     if (e.key === "Escape") {
  //       setCommandPaletteOpen(false)
  //     }
  //   }

  //   window.addEventListener("keydown", handleKeyDown)
  //   return () => window.removeEventListener("keydown", handleKeyDown)
  // }, [])

  return (
    <div className="relative h-full w-full overflow-hidden">
      <ViewportCanvas />
      <ViewportToolbar />
      <BottomToolbar onOpenCommandPalette={() => {}} />
      {/* <CommandPalette open={commandPaletteOpen} onOpenChange={setCommandPaletteOpen} /> */}
    </div>
  )
}
