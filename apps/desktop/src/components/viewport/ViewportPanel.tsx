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
} from "@hugeicons/core-free-icons"
import { HugeiconsIcon } from "@hugeicons/react"
import { Button, Tooltip, TooltipContent, TooltipTrigger } from "@cadhy/ui"
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
    setViewMode(mode)
    try {
      await viewportApi.setViewMode(mode)
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
    <div className="absolute right-2 top-2 flex flex-col gap-1 rounded-lg bg-card/90 p-1.5 backdrop-blur-md border border-border/50 shadow-lg z-10">
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

function ViewportCanvas() {
  const containerRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const setSize = useViewportStore((s) => s.setSize)
  const setCursorPosition = useViewportStore((s) => s.setCursorPosition)
  const setFps = useViewportStore((s) => s.setFps)
  const [isInitialized, setIsInitialized] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Ref to prevent duplicate initialization (React StrictMode calls useEffect twice)
  const initializingRef = useRef(false)

  // Camera control state
  const isDragging = useRef(false)
  const dragMode = useRef<"orbit" | "pan" | null>(null)
  const lastPointer = useRef<{ x: number; y: number }>({ x: 0, y: 0 })
  const animationFrameRef = useRef<number | null>(null)

  // State to track rendering mode
  const [isNativeMode, setIsNativeMode] = useState(false)

  // Initialize viewport
  useEffect(() => {
    let mounted = true

    const init = async () => {
      // Prevent duplicate initialization
      if (initializingRef.current) {
        console.log("[Viewport] Already initializing, skipping duplicate call")
        return
      }

      if (!containerRef.current || !canvasRef.current) return

      const rect = containerRef.current.getBoundingClientRect()
      const width = Math.floor(rect.width)
      const height = Math.floor(rect.height)

      if (width === 0 || height === 0) {
        setTimeout(init, 100)
        return
      }

      initializingRef.current = true

      canvasRef.current.width = width
      canvasRef.current.height = height

      try {
        // Try native mode first (60+ FPS)
        // Pass the position of the viewport within the main window
        const x = Math.floor(rect.left)
        const y = Math.floor(rect.top)

        console.log(`[Viewport] Attempting NATIVE mode ${width}x${height} at (${x}, ${y})...`)
        const isNative = await viewportApi.initNative(width, height, x, y)

        if (mounted) {
          setIsNativeMode(isNative)
          setIsInitialized(true)
          setSize(width, height)

          if (isNative) {
            // In native mode, set high FPS indicator
            setFps(60, 16.67)
          }

          console.log(
            `[Viewport] Initialized ${width}x${height} - ${isNative ? "NATIVE" : "OFFSCREEN"} mode`,
          )
        }
      } catch (e) {
        console.error("[Viewport] Native init failed, falling back to offscreen:", e)

        // Fallback to offscreen mode
        try {
          await viewportApi.init(width, height)
          if (mounted) {
            setIsNativeMode(false)
            setIsInitialized(true)
            setSize(width, height)
            console.log(`[Viewport] Initialized ${width}x${height} - OFFSCREEN mode (fallback)`)
          }
        } catch (e2) {
          if (mounted) {
            setError(e2 instanceof Error ? e2.message : String(e2))
          }
        }
      }
    }

    init()
    return () => {
      mounted = false
    }
  }, [setSize, setFps])

  // Render loop for offscreen mode (only runs when NOT in native mode)
  useEffect(() => {
    // In native mode, the Rust render thread handles rendering
    if (isNativeMode) {
      console.log("[Viewport] Native mode active - skipping JS render loop")
      return
    }

    if (!isInitialized || !canvasRef.current) return

    const canvas = canvasRef.current
    const ctx = canvas.getContext("2d", { alpha: false })
    if (!ctx) return

    let running = true
    let frameCount = 0
    let lastFpsUpdate = performance.now()

    // Pre-allocate ImageData for reuse (avoids GC pressure)
    let cachedImageData: ImageData | null = null
    let cachedWidth = 0
    let cachedHeight = 0

    // Fast base64 decode
    const decodeBase64Fast = (base64: string): Uint8ClampedArray => {
      const binaryString = atob(base64)
      const len = binaryString.length
      const bytes = new Uint8ClampedArray(len)
      // Unrolled loop for better performance
      for (let i = 0; i < len; i += 4) {
        bytes[i] = binaryString.charCodeAt(i)
        bytes[i + 1] = binaryString.charCodeAt(i + 1)
        bytes[i + 2] = binaryString.charCodeAt(i + 2)
        bytes[i + 3] = binaryString.charCodeAt(i + 3)
      }
      return bytes
    }

    const render = async () => {
      if (!running) return

      try {
        const dataUri = await viewportApi.renderFrame()

        // Parse: data:image/rgba;width=W;height=H;base64,DATA
        const match = dataUri.match(/^data:image\/rgba;width=(\d+);height=(\d+);base64,(.+)$/)
        if (match && canvas.width > 0 && canvas.height > 0) {
          const width = Number.parseInt(match[1], 10)
          const height = Number.parseInt(match[2], 10)
          const base64Data = match[3]

          // Decode base64 to RGBA
          const rgbaData = decodeBase64Fast(base64Data)

          // Reuse ImageData if dimensions match
          if (cachedWidth !== width || cachedHeight !== height) {
            cachedImageData = new ImageData(width, height)
            cachedWidth = width
            cachedHeight = height
          }

          cachedImageData!.data.set(rgbaData)
          ctx.putImageData(cachedImageData!, 0, 0)
        }

        // FPS calculation
        frameCount++
        const now = performance.now()
        if (now - lastFpsUpdate >= 1000) {
          setFps(frameCount, frameCount > 0 ? 1000 / frameCount : 0)
          frameCount = 0
          lastFpsUpdate = now
        }
      } catch (e) {
        console.error("[Viewport] Render error:", e)
      }

      if (running) {
        animationFrameRef.current = requestAnimationFrame(render)
      }
    }

    render()

    return () => {
      running = false
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current)
      }
    }
  }, [isInitialized, isNativeMode, setFps])

  // Handle resize
  useEffect(() => {
    if (!containerRef.current || !isInitialized) return

    const observer = new ResizeObserver(async (entries) => {
      for (const entry of entries) {
        const rect = entry.target.getBoundingClientRect()
        const w = Math.floor(rect.width)
        const h = Math.floor(rect.height)
        const x = Math.floor(rect.left)
        const y = Math.floor(rect.top)

        if (w > 0 && h > 0 && canvasRef.current) {
          canvasRef.current.width = w
          canvasRef.current.height = h
          setSize(w, h)
          try {
            if (isNativeMode) {
              // In native mode, update the native window bounds
              await viewportApi.updateBounds(x, y, w, h)
            } else {
              // In offscreen mode, just resize the renderer
              await viewportApi.resize(w, h)
            }
          } catch (e) {
            console.error("[Viewport] Resize error:", e)
          }
        }
      }
    })

    observer.observe(containerRef.current)
    return () => observer.disconnect()
  }, [isInitialized, isNativeMode, setSize])

  // In native mode, listen for window move events to reposition viewport
  useEffect(() => {
    if (!isNativeMode || !containerRef.current) return

    // Function to update viewport bounds
    const updateViewportBounds = async () => {
      if (!containerRef.current) return
      const rect = containerRef.current.getBoundingClientRect()
      const x = Math.floor(rect.left)
      const y = Math.floor(rect.top)
      const w = Math.floor(rect.width)
      const h = Math.floor(rect.height)

      if (w > 0 && h > 0) {
        try {
          await viewportApi.updateBounds(x, y, w, h)
        } catch (e) {
          console.error("[Viewport] Update bounds error:", e)
        }
      }
    }

    // Listen for scroll/layout changes that might affect position
    const handleScroll = () => updateViewportBounds()
    window.addEventListener("scroll", handleScroll, true)

    // Use an interval to periodically sync position (handles window moves)
    const intervalId = setInterval(updateViewportBounds, 100)

    return () => {
      window.removeEventListener("scroll", handleScroll, true)
      clearInterval(intervalId)
    }
  }, [isNativeMode])

  // Camera control handlers
  const handlePointerDown = useCallback((e: PointerEvent<HTMLDivElement>) => {
    if (e.button === 1 || (e.button === 0 && e.altKey)) {
      e.preventDefault()
      isDragging.current = true
      lastPointer.current = { x: e.clientX, y: e.clientY }
      dragMode.current = e.shiftKey ? "pan" : "orbit"
      ;(e.target as HTMLElement).setPointerCapture(e.pointerId)
    }
  }, [])

  const handlePointerMove = useCallback(
    async (e: PointerEvent<HTMLDivElement>) => {
      if (containerRef.current) {
        const rect = containerRef.current.getBoundingClientRect()
        const x = e.clientX - rect.left
        const y = e.clientY - rect.top
        const ndcX = (x / rect.width) * 2 - 1
        const ndcY = -((y / rect.height) * 2 - 1)
        setCursorPosition(ndcX * 10, ndcY * 10, 0)
      }

      if (!isDragging.current) return

      const deltaX = e.clientX - lastPointer.current.x
      const deltaY = e.clientY - lastPointer.current.y
      lastPointer.current = { x: e.clientX, y: e.clientY }

      try {
        if (dragMode.current === "orbit") {
          // Use native commands for better performance
          await viewportApi.orbitNative(deltaX, deltaY)
        } else if (dragMode.current === "pan") {
          await viewportApi.panNative(deltaX, deltaY)
        }
      } catch (e) {
        console.error("Camera control error:", e)
      }
    },
    [setCursorPosition],
  )

  const handlePointerUp = useCallback((e: PointerEvent<HTMLDivElement>) => {
    if (isDragging.current) {
      isDragging.current = false
      dragMode.current = null
      ;(e.target as HTMLElement).releasePointerCapture(e.pointerId)
    }
  }, [])

  const handleWheel = useCallback(async (e: WheelEvent<HTMLDivElement>) => {
    e.preventDefault()
    const delta = e.deltaY > 0 ? 100 : -100

    try {
      await viewportApi.zoomNative(delta)
    } catch (e) {
      console.error("Zoom error:", e)
    }
  }, [])

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault()
  }, [])

  return (
    <div
      ref={containerRef}
      role="application"
      aria-label="3D Viewport"
      className="absolute inset-0 cursor-crosshair touch-none bg-[#1a1a1a]"
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerLeave={handlePointerUp}
      onWheel={handleWheel}
      onContextMenu={handleContextMenu}
    >
      {!isInitialized && !error && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center pointer-events-none">
            <div className="mb-4 opacity-10">
              <HugeiconsIcon icon={CubeIcon} size={80} />
            </div>
            <p className="text-sm text-muted-foreground/60">Initializing wgpu...</p>
          </div>
        </div>
      )}
      {error && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center pointer-events-none">
            <p className="text-sm text-red-400/80">{error}</p>
          </div>
        </div>
      )}
      {/* Viewport Corners - 16px Rounded Effect with seamless border integration */}
      <div className="absolute inset-0 pointer-events-none z-20">
        {/* Top Left */}
        <svg width="16" height="16" viewBox="0 0 16 16" className="absolute top-0 left-0 fill-card">
          <path d="M0 0 L16 0 Q0 0 0 16 L0 0 Z" />
          <path d="M16 0 Q0 0 0 16" fill="none" stroke="currentColor" strokeWidth="0.5" className="text-border/30" />
        </svg>
        {/* Top Right */}
        <svg width="16" height="16" viewBox="0 0 16 16" className="absolute top-0 right-0 fill-card">
          <path d="M16 0 L0 0 Q16 0 16 16 L16 0 Z" />
          <path d="M0 0 Q16 0 16 16" fill="none" stroke="currentColor" strokeWidth="0.5" className="text-border/30" />
        </svg>
        {/* Bottom Left */}
        <svg width="16" height="16" viewBox="0 0 16 16" className="absolute bottom-0 left-0 fill-card">
          <path d="M0 16 L16 16 Q0 16 0 0 L0 16 Z" />
          <path d="M16 16 Q0 16 0 0" fill="none" stroke="currentColor" strokeWidth="0.5" className="text-border/30" />
        </svg>
        {/* Bottom Right */}
        <svg width="16" height="16" viewBox="0 0 16 16" className="absolute bottom-0 right-0 fill-card">
          <path d="M16 16 L0 16 Q16 16 16 0 L16 16 Z" />
          <path d="M0 16 Q16 16 16 0" fill="none" stroke="currentColor" strokeWidth="0.5" className="text-border/30" />
        </svg>
      </div>

      {/* Canvas is only used for offscreen mode - hidden in native mode */}
      <canvas
        ref={canvasRef}
        className={`absolute inset-0 w-full h-full ${isInitialized && !isNativeMode ? "" : "opacity-0"}`}
        style={{ imageRendering: "auto", pointerEvents: isNativeMode ? "none" : "auto" }}
      />
    </div>
  )
}

// ============================================================================
// BOTTOM TOOLBAR - Horizontal controls bar
// ============================================================================

function BottomToolbar() {
  const [maxEvents, setMaxEvents] = useState(64)
  const [foliation, setFoliation] = useState(0)

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
        >
          <path d="M10 0 L10 10 L0 10 Q10 10 10 0" />
          <path d="M10 0 Q10 10 0 10" fill="none" stroke="currentColor" strokeWidth="0.5" className="text-border/20" />
        </svg>

        {/* Main toolbar content */}
        <div className="flex items-center h-9 px-3 bg-card border-t border-border/20 gap-3 rounded-t-[var(--radius)]">
          {/* Layout toggle button */}
          <button
            type="button"
            className="p-1 text-muted-foreground hover:text-foreground transition-colors rounded hover:bg-accent"
            title="Switch to horizontal layout"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M3 8h10M10 5l3 3-3 3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>

          {/* Separator */}
          <div className="w-px h-4 bg-border/30" />

          {/* Max Events slider */}
          <div className="flex items-center gap-2">
            <span className="text-[10px] text-muted-foreground uppercase tracking-wider whitespace-nowrap">
              Max Events
            </span>
            <div className="relative w-20 flex items-center">
              <input
                type="range"
                min="8"
                max="256"
                step="8"
                value={maxEvents}
                onChange={(e) => setMaxEvents(Number(e.target.value))}
                className="w-full h-1 bg-muted rounded-full appearance-none cursor-pointer accent-primary"
              />
            </div>
            <span className="text-[10px] text-muted-foreground font-medium w-6 text-right">
              {maxEvents}
            </span>
          </div>

          {/* Separator */}
          <div className="w-px h-4 bg-border/30" />

          {/* Foliation slider */}
          <div className="flex items-center gap-2">
            <span className="text-[10px] text-muted-foreground uppercase tracking-wider whitespace-nowrap">
              Foliation
            </span>
            <div className="relative w-20 flex items-center">
              <input
                type="range"
                min="-100"
                max="100"
                step="5"
                value={foliation}
                onChange={(e) => setFoliation(Number(e.target.value))}
                className="w-full h-1 bg-muted rounded-full appearance-none cursor-pointer accent-primary"
              />
            </div>
            <span className="text-[10px] text-muted-foreground font-medium w-6 text-right">
              {foliation}
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
        >
          <path d="M0 0 Q0 10 10 10 L0 10 L0 0" />
          <path d="M0 0 Q0 10 10 10" fill="none" stroke="currentColor" strokeWidth="0.5" className="text-border/20" />
        </svg>
      </div>
    </div>
  )
}

// ============================================================================
// MAIN VIEWPORT PANEL
// ============================================================================

export function ViewportPanel() {
  return (
    <div className="relative h-full w-full overflow-hidden">
      <ViewportCanvas />
      <ViewportToolbar />
      <BottomToolbar />
    </div>
  )
}
