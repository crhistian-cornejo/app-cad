/**
 * ViewportPanel - CADHY 3D Viewport with wgpu Rendering
 *
 * Uses cadhy-bridge for wgpu rendering with optimized render loop.
 *
 * Camera Controls (Plasticity-style):
 * - Left Click + Drag: Orbit camera
 * - Alt/Option + Left Click + Drag: Orbit (alternative for trackpads)
 * - Middle Click + Drag: Pan camera
 * - Shift + Left Click + Drag: Pan camera
 * - Right Click + Drag: Pan camera
 * - Scroll: Zoom in/out
 */

import { useCallback, useEffect, useRef, useState, type PointerEvent, type WheelEvent } from "react"
import type { PickResult } from "@/lib/tauri"
import {
  Add01Icon,
  Camera01Icon,
  CubeIcon,
  GridIcon,
  MaximizeScreenIcon,
  Search01Icon,
  ThreeDViewIcon,
  SquareIcon,
  Home01Icon,
} from "@hugeicons/core-free-icons"
import { HugeiconsIcon } from "@hugeicons/react"
import { Button, cn, Kbd, Slider, Tooltip, TooltipContent, TooltipTrigger } from "@cadhy/ui"
import { useViewportStore } from "@/stores/viewport-store"
import { useLayoutStore } from "@/stores/layout-store"
import { useSceneStore } from "@/stores/scene-store"
import { sceneApiDirect, wgpuOverlayApi } from "@/lib/tauri"

// Detect platform for keyboard shortcuts
const isMac = typeof navigator !== "undefined" && navigator.platform.toUpperCase().indexOf("MAC") >= 0

// ============================================================================
// VIEWPORT TOOLBAR
// ============================================================================

function ViewportToolbar() {
  const { viewMode, setViewMode, frameAll, resetCamera, projection, setProjection } = useViewportStore()
  const [isAddingCube, setIsAddingCube] = useState(false)
  const [gizmoMode, setGizmoMode] = useState<"translate" | "rotate" | "scale">("translate")

  const handleSetGizmoMode = async (mode: "translate" | "rotate" | "scale") => {
    try {
      await wgpuOverlayApi.setGizmoMode(mode)
      setGizmoMode(mode)
    } catch (e) {
      console.error("Failed to set gizmo mode:", e)
    }
  }

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

  const handleToggleProjection = () => {
    const newProjection = projection === "perspective" ? "orthographic" : "perspective"
    setProjection(newProjection)
    // TODO: Send to backend when implemented
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

      {/* Gizmo Mode */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant={gizmoMode === "translate" ? "secondary" : "ghost"}
            size="icon"
            className="size-8"
            onClick={() => handleSetGizmoMode("translate")}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M12 2v20M2 12h20M12 2l3 3M12 2l-3 3M12 22l3-3M12 22l-3-3M2 12l3 3M2 12l3-3M22 12l-3 3M22 12l-3-3" />
            </svg>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Move (G)</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant={gizmoMode === "rotate" ? "secondary" : "ghost"}
            size="icon"
            className="size-8"
            onClick={() => handleSetGizmoMode("rotate")}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="8" />
              <path d="M20 12a8 8 0 01-8 8M4 12a8 8 0 018-8M12 4l2 2M12 4l-2 2" />
            </svg>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Rotate (R)</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant={gizmoMode === "scale" ? "secondary" : "ghost"}
            size="icon"
            className="size-8"
            onClick={() => handleSetGizmoMode("scale")}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="4" y="4" width="16" height="16" rx="1" />
              <path d="M4 4l4 4M20 4l-4 4M4 20l4-4M20 20l-4-4" />
            </svg>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Scale (S)</TooltipContent>
      </Tooltip>

      <div className="my-1 h-px bg-border/50" />

      {/* View Mode */}
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

      {/* Projection Toggle */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant={projection === "orthographic" ? "secondary" : "ghost"}
            size="icon"
            className="size-8"
            onClick={handleToggleProjection}
          >
            <HugeiconsIcon icon={projection === "perspective" ? ThreeDViewIcon : SquareIcon} size={18} />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">
          {projection === "perspective" ? "Switch to Orthographic (5)" : "Switch to Perspective (5)"}
        </TooltipContent>
      </Tooltip>

      <div className="my-1 h-px bg-border/50" />

      {/* Camera Controls */}
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
            <HugeiconsIcon icon={Home01Icon} size={18} />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Reset Camera</TooltipContent>
      </Tooltip>
    </div>
  )
}

// ============================================================================
// AXES GIZMO (Orientation Cube)
// ============================================================================

function AxesGizmo() {
  const { showAxes } = useLayoutStore()

  if (!showAxes) return null

  return (
    <div className="absolute left-4 bottom-16 z-10 pointer-events-none">
      <svg width="80" height="80" viewBox="-40 -40 80 80" className="opacity-80">
        {/* Y axis (up - green) */}
        <line x1="0" y1="0" x2="0" y2="-30" stroke="#22c55e" strokeWidth="2.5" strokeLinecap="round" />
        <polygon points="0,-35 -4,-28 4,-28" fill="#22c55e" />
        <text x="0" y="-38" textAnchor="middle" fill="#22c55e" fontSize="10" fontWeight="bold">Y</text>

        {/* X axis (right - red) */}
        <line x1="0" y1="0" x2="26" y2="15" stroke="#ef4444" strokeWidth="2.5" strokeLinecap="round" />
        <polygon points="32,18 24,20 26,12" fill="#ef4444" />
        <text x="36" y="22" textAnchor="middle" fill="#ef4444" fontSize="10" fontWeight="bold">X</text>

        {/* Z axis (forward - blue) */}
        <line x1="0" y1="0" x2="-26" y2="15" stroke="#3b82f6" strokeWidth="2.5" strokeLinecap="round" />
        <polygon points="-32,18 -26,12 -24,20" fill="#3b82f6" />
        <text x="-36" y="22" textAnchor="middle" fill="#3b82f6" fontSize="10" fontWeight="bold">Z</text>

        {/* Center dot */}
        <circle cx="0" cy="0" r="3" fill="currentColor" className="text-muted-foreground/50" />
      </svg>
    </div>
  )
}

// ============================================================================
// VIEW PRESETS (Quick camera views)
// ============================================================================

function ViewPresetsToolbar() {
  const handleSetView = async (view: string) => {
    // TODO: Implement view presets in backend
    console.log(`Set view: ${view}`)
  }

  return (
    <div className="absolute left-2 top-2 flex flex-col gap-1 rounded-lg bg-card/90 p-1.5 backdrop-blur-md border border-border/20 shadow-lg z-10">
      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="icon" className="size-7" onClick={() => handleSetView("front")}>
            <span className="text-[10px] font-bold">F</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">Front View (1)</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="icon" className="size-7" onClick={() => handleSetView("right")}>
            <span className="text-[10px] font-bold">R</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">Right View (3)</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="icon" className="size-7" onClick={() => handleSetView("top")}>
            <span className="text-[10px] font-bold">T</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">Top View (7)</TooltipContent>
      </Tooltip>

      <div className="my-0.5 h-px bg-border/50" />

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="icon" className="size-7" onClick={() => handleSetView("back")}>
            <span className="text-[9px] font-medium text-muted-foreground">B</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">Back View (Ctrl+1)</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="icon" className="size-7" onClick={() => handleSetView("left")}>
            <span className="text-[9px] font-medium text-muted-foreground">L</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">Left View (Ctrl+3)</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="icon" className="size-7" onClick={() => handleSetView("bottom")}>
            <span className="text-[9px] font-medium text-muted-foreground">Bo</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">Bottom View (Ctrl+7)</TooltipContent>
      </Tooltip>
    </div>
  )
}

// ============================================================================
// VIEWPORT CANVAS with optimized render loop
// ============================================================================

// Note: wgpu window fills the entire container. Toolbars overlay on top with z-index.

// Gizmo axis type for drag tracking
type GizmoAxis = "x" | "y" | "z" | "xy" | "xz" | "yz" | "all" | "none"

function ViewportCanvas() {
  const containerRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const animationFrameRef = useRef<number | undefined>(undefined)
  const isDragging = useRef(false)
  const dragMode = useRef<"orbit" | "pan" | "select" | "gizmo">("orbit")
  const lastMousePos = useRef({ x: 0, y: 0 })
  const clickStartPos = useRef({ x: 0, y: 0 })
  const hasMoved = useRef(false)
  const gizmoAxis = useRef<GizmoAxis>("none")
  const gizmoDragStart = useRef({ x: 0, y: 0 })

  const { isInitialized, setInitialized, frameAll, resetCamera, setCursorPosition } = useViewportStore()
  const { select, deselectAll, translateSelection, selection } = useSceneStore()

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
  // Plasticity-style controls:
  // - Middle click + drag: Orbit (Plasticity default)
  // - Ctrl + Left click (Windows) / Option + Left click (macOS): Orbit
  // - Right click + drag: Pan
  // - Shift + Left click: Pan
  // - Left click on gizmo: Gizmo drag
  // - Left click on object/background: Select
  // - Scroll: Zoom
  const handlePointerDown = useCallback(async (e: PointerEvent<HTMLDivElement>) => {
    // Button 0 = left, 1 = middle, 2 = right
    const isLeftButton = e.button === 0
    const isMiddleButton = e.button === 1
    const isRightButton = e.button === 2

    if (isLeftButton || isMiddleButton || isRightButton) {
      e.preventDefault()
      e.stopPropagation()
      isDragging.current = true
      hasMoved.current = false
      lastMousePos.current = { x: e.clientX, y: e.clientY }
      clickStartPos.current = { x: e.clientX, y: e.clientY }
      gizmoAxis.current = "none"

      // Determine drag mode:
      // ORBIT: Middle mouse, Ctrl+Left (Windows), Alt/Option+Left (macOS/trackpad)
      // PAN: Right mouse, Shift+Left
      // SELECT/GIZMO: Left click without modifiers - check if clicking on gizmo
      const isOrbit = isMiddleButton ||
        (isLeftButton && (e.ctrlKey || e.altKey || e.metaKey))
      const isPan = isRightButton || (isLeftButton && e.shiftKey)

      if (isOrbit) {
        dragMode.current = "orbit"
      } else if (isPan) {
        dragMode.current = "pan"
      } else if (isLeftButton) {
        // Plain left click - check if clicking on gizmo
        // Only check for gizmo if there's a selection
        if (selection.ids.length > 0) {
          try {
            const x = Math.round(e.clientX)
            const y = Math.round(e.clientY)
            const pickResult = await wgpuOverlayApi.pick(x, y)

            if (pickResult.type === "gizmo") {
              // Start gizmo drag
              dragMode.current = "gizmo"
              gizmoAxis.current = pickResult.axis as GizmoAxis
              gizmoDragStart.current = { x: e.clientX, y: e.clientY }
              console.log("[Viewport] Started gizmo drag on axis:", pickResult.axis)
            } else {
              // Not gizmo - start selection mode
              dragMode.current = "select"
            }
          } catch (err) {
            console.error("[Viewport] Pick error on pointer down:", err)
            dragMode.current = "select"
          }
        } else {
          // No selection - just do select mode
          dragMode.current = "select"
        }
      }

      // Capture pointer to receive events even when cursor leaves element
      if (containerRef.current) {
        containerRef.current.setPointerCapture(e.pointerId)
      }
    }
  }, [selection.ids.length])

  const handlePointerMove = useCallback((e: PointerEvent<HTMLDivElement>) => {
    // Always update cursor position for status bar display
    // For now, show screen coordinates (0,0 at top-left)
    // TODO: Convert to world coordinates via raycasting
    if (containerRef.current) {
      const rect = containerRef.current.getBoundingClientRect()
      const x = e.clientX - rect.left
      const y = e.clientY - rect.top
      // Display as normalized coordinates for now (can be enhanced later with world coords)
      setCursorPosition(x, y, 0)
    }

    if (!isDragging.current || !isInitialized) return

    const deltaX = e.clientX - lastMousePos.current.x
    const deltaY = e.clientY - lastMousePos.current.y
    lastMousePos.current = { x: e.clientX, y: e.clientY }

    // Skip if no movement
    if (deltaX === 0 && deltaY === 0) return

    // Check if we've moved significantly from click start
    const totalDeltaX = e.clientX - clickStartPos.current.x
    const totalDeltaY = e.clientY - clickStartPos.current.y
    const distanceMoved = Math.sqrt(totalDeltaX * totalDeltaX + totalDeltaY * totalDeltaY)

    // If we're in select mode but moved more than 5 pixels, switch to orbit
    if (dragMode.current === "select" && distanceMoved > 5) {
      dragMode.current = "orbit"
      hasMoved.current = true
    }

    // Fire and forget - don't await to avoid blocking the event loop
    if (dragMode.current === "orbit") {
      hasMoved.current = true
      wgpuOverlayApi.orbit(deltaX, deltaY).catch(console.error)
    } else if (dragMode.current === "pan") {
      hasMoved.current = true
      wgpuOverlayApi.pan(deltaX, deltaY).catch(console.error)
    } else if (dragMode.current === "gizmo" && gizmoAxis.current !== "none") {
      // Handle gizmo drag - translate selection along axis
      hasMoved.current = true
      const sensitivity = 0.01

      // Calculate delta based on gizmo axis
      // For simplicity: X axis uses screen X movement, Y axis uses screen Y movement, Z uses screen Y
      let dx = 0, dy = 0, dz = 0

      switch (gizmoAxis.current) {
        case "x":
          dx = deltaX * sensitivity
          break
        case "y":
          dy = -deltaY * sensitivity // Screen Y is inverted
          break
        case "z":
          dz = deltaY * sensitivity // Use screen Y for Z depth
          break
        case "xy":
          dx = deltaX * sensitivity
          dy = -deltaY * sensitivity
          break
        case "xz":
          dx = deltaX * sensitivity
          dz = deltaY * sensitivity
          break
        case "yz":
          dy = -deltaY * sensitivity
          dz = deltaX * sensitivity
          break
        case "all":
          // Uniform scale or move in all directions
          const avgDelta = (deltaX - deltaY) * sensitivity
          dx = avgDelta
          dy = avgDelta
          dz = avgDelta
          break
      }

      if (dx !== 0 || dy !== 0 || dz !== 0) {
        translateSelection(dx, dy, dz).catch(console.error)
      }
    }
    // In select mode, don't do anything until mouse up
  }, [isInitialized, setCursorPosition, translateSelection])

  const handlePointerUp = useCallback(async (e: PointerEvent<HTMLDivElement>) => {
    if (isDragging.current) {
      const wasSelect = dragMode.current === "select" && !hasMoved.current

      isDragging.current = false
      if (containerRef.current) {
        containerRef.current.releasePointerCapture(e.pointerId)
      }

      // If this was a click (select mode with no movement), perform picking
      if (wasSelect) {
        try {
          const x = Math.round(clickStartPos.current.x)
          const y = Math.round(clickStartPos.current.y)
          const pickResult = await wgpuOverlayApi.pick(x, y)

          if (pickResult.type === "object") {
            // Select the picked object (uses store method to sync state)
            await select([pickResult.id], e.shiftKey)
            console.log("[Viewport] Selected object:", pickResult.id)
          } else if (pickResult.type === "gizmo") {
            // Clicked on gizmo axis - this was already a gizmo drag that ended
            console.log("[Viewport] Gizmo drag ended on axis:", pickResult.axis)
          } else {
            // Clicked on background - deselect all (unless shift is held)
            if (!e.shiftKey) {
              await deselectAll()
              console.log("[Viewport] Deselected all")
            }
          }
        } catch (err) {
          console.error("[Viewport] Pick error:", err)
        }
      }
    }
  }, [select, deselectAll])

  // Handle pointer leaving the window
  const handlePointerLeave = useCallback((e: PointerEvent<HTMLDivElement>) => {
    // Don't stop dragging if pointer is captured
    if (isDragging.current && !containerRef.current?.hasPointerCapture(e.pointerId)) {
      isDragging.current = false
    }
  }, [])

  const handleWheel = useCallback((e: WheelEvent<HTMLDivElement>) => {
    if (!isInitialized) return
    e.preventDefault()

    // Normalize scroll delta across different input devices
    // deltaY is typically ~100 for mouse wheel, smaller for trackpad
    const delta = e.deltaY
    wgpuOverlayApi.zoom(-delta).catch(console.error)
  }, [isInitialized])

  // No render loop needed - wgpu runs continuously in Rust

  return (
    <div
      ref={containerRef}
      role="application"
      aria-label="3D Viewport"
      // Inverted architecture: transparent so wgpu content shows through
      // Use rgba with 0.001 alpha to ensure pointer events work on all platforms
      className="absolute inset-0 cursor-crosshair touch-none"
      style={{
        backgroundColor: "rgba(0, 0, 0, 0.001)", // Near-transparent but ensures pointer events work
        pointerEvents: "auto",
      }}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerLeave={handlePointerLeave}
      onPointerCancel={handlePointerUp}
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
          <path d="M24 0 L0 0 L0 24 L1 24 L1 13 Q1 1 13 1 L24 1 Z" />
          <path
            d="M24 1 L13 1 Q1 1 1 13 L1 24"
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
          <path d="M0 0 L24 0 L24 24 L23 24 L23 13 Q23 1 11 1 L0 1 Z" />
          <path
            d="M0 1 L11 1 Q23 1 23 13 L23 24"
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
          <path d="M24 24 L0 24 L0 0 L1 0 L1 11 Q1 23 11 23 L24 23 Z" />
          <path
            d="M24 23 L11 23 Q1 23 1 11 L1 0"
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
          <path d="M0 24 L24 24 L24 0 L23 0 L23 11 Q23 23 13 23 L0 23 Z" />
          <path
            d="M0 23 L13 23 Q23 23 23 11 L23 0"
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
  const { projection, setProjection } = useViewportStore()

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't capture if user is typing in an input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return
      }

      // Numpad 5 or regular 5: Toggle perspective/orthographic
      if (e.key === "5") {
        e.preventDefault()
        setProjection(projection === "perspective" ? "orthographic" : "perspective")
      }
    }

    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [projection, setProjection])

  return (
    <div className="relative h-full w-full overflow-hidden">
      <ViewportCanvas />
      <ViewPresetsToolbar />
      <ViewportToolbar />
      <AxesGizmo />
      <BottomToolbar onOpenCommandPalette={() => {}} />
    </div>
  )
}
