/**
 * ViewportPanel - CADHY 3D Viewport with wgpu
 */

import { useCallback, useEffect, useRef, useState, type PointerEvent, type WheelEvent } from "react"
import {
  Add01Icon,
  Camera01Icon,
  CubeIcon,
  GridIcon,
  MaximizeScreenIcon,
  Drag01Icon,
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
    // TODO: Call backend to change view mode
  }

  return (
    <div className="absolute right-2 top-2 flex flex-col gap-1 rounded-lg bg-card/90 p-1.5 backdrop-blur-md border border-border/50 shadow-lg">
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
          <Button variant="ghost" size="icon" className="size-8" onClick={resetCamera}>
            <HugeiconsIcon icon={Camera01Icon} size={18} />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Reset Camera</TooltipContent>
      </Tooltip>
    </div>
  )
}

// ============================================================================
// VIEWPORT CANVAS
// ============================================================================

function ViewportCanvas() {
  const containerRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const setSize = useViewportStore((s) => s.setSize)
  const setCursorPosition = useViewportStore((s) => s.setCursorPosition)
  const setFps = useViewportStore((s) => s.setFps)
  const [isInitialized, setIsInitialized] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Camera control state
  const isDragging = useRef(false)
  const dragMode = useRef<"orbit" | "pan" | null>(null)
  const lastPointer = useRef<{ x: number; y: number }>({ x: 0, y: 0 })

  // FPS tracking
  const frameTimesRef = useRef<number[]>([])
  const lastFrameTimeRef = useRef(performance.now())

  // Initialize renderer
  useEffect(() => {
    const initRenderer = async () => {
      if (!containerRef.current) return

      const rect = containerRef.current.getBoundingClientRect()
      const width = Math.floor(rect.width)
      const height = Math.floor(rect.height)

      if (width === 0 || height === 0) {
        setTimeout(initRenderer, 100)
        return
      }

      try {
        const success = await viewportApi.init(width, height)
        if (success) {
          setIsInitialized(true)
          setError(null)
        } else {
          setError("Renderer initialization failed")
        }
      } catch (e) {
        console.error("[Viewport] Init error:", e)
        setError(e instanceof Error ? e.message : String(e))
      }
    }

    initRenderer()
  }, [])

  // Render loop with FPS tracking
  useEffect(() => {
    if (!isInitialized || !canvasRef.current) return

    const canvas = canvasRef.current
    const ctx = canvas.getContext("2d")
    if (!ctx) return

    let running = true
    let frameId: number | null = null
    let errorCount = 0
    const MAX_ERRORS = 5

    const updateFps = () => {
      const now = performance.now()
      const frameTimes = frameTimesRef.current

      // Add current frame time
      frameTimes.push(now)

      // Keep only last 60 frames
      while (frameTimes.length > 60) {
        frameTimes.shift()
      }

      // Calculate FPS from frame times
      if (frameTimes.length >= 2) {
        const elapsed = frameTimes[frameTimes.length - 1] - frameTimes[0]
        const fps = Math.round((frameTimes.length - 1) / (elapsed / 1000))
        const frameTime = Math.round(elapsed / (frameTimes.length - 1) * 100) / 100
        setFps(fps, frameTime)
      }
    }

    const render = async () => {
      if (!running) return

      const frameStart = performance.now()

      try {
        const frameData = await viewportApi.renderFrame()
        errorCount = 0

        if (frameData && ctx && running) {
          const match = frameData.match(/width=(\d+);height=(\d+);base64,(.+)/)
          if (match) {
            const width = parseInt(match[1], 10)
            const height = parseInt(match[2], 10)
            const base64 = match[3]

            if (width <= 0 || height <= 0) {
              if (running) frameId = requestAnimationFrame(render)
              return
            }

            const binary = atob(base64)
            const bytes = new Uint8Array(binary.length)
            for (let i = 0; i < binary.length; i++) {
              bytes[i] = binary.charCodeAt(i)
            }

            const expectedLength = width * height * 4
            if (bytes.length !== expectedLength) {
              if (running) frameId = requestAnimationFrame(render)
              return
            }

            const imageData = new ImageData(new Uint8ClampedArray(bytes.buffer), width, height)

            if (canvas.width !== width || canvas.height !== height) {
              canvas.width = width
              canvas.height = height
            }

            ctx.putImageData(imageData, 0, 0)
            updateFps()
          }
        }
      } catch (e) {
        errorCount++
        if (errorCount > MAX_ERRORS) {
          await new Promise(resolve => setTimeout(resolve, 100))
          errorCount = 0
        }
      }

      if (running) {
        frameId = requestAnimationFrame(render)
      }
    }

    frameId = requestAnimationFrame(render)

    return () => {
      running = false
      if (frameId !== null) cancelAnimationFrame(frameId)
    }
  }, [isInitialized, setFps])

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
    (e: PointerEvent<HTMLDivElement>) => {
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

      if (dragMode.current === "orbit") {
        viewportApi.orbit(deltaX, deltaY)
      } else if (dragMode.current === "pan") {
        viewportApi.pan(deltaX, deltaY)
      }
    },
    [setCursorPosition]
  )

  const handlePointerUp = useCallback((e: PointerEvent<HTMLDivElement>) => {
    if (isDragging.current) {
      isDragging.current = false
      dragMode.current = null
      ;(e.target as HTMLElement).releasePointerCapture(e.pointerId)
    }
  }, [])

  const handleWheel = useCallback((e: WheelEvent<HTMLDivElement>) => {
    e.preventDefault()
    const delta = e.deltaY > 0 ? 100 : -100
    viewportApi.zoom(delta)
  }, [])

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault()
  }, [])

  // Handle resize with debounce
  useEffect(() => {
    if (!containerRef.current) return

    let resizeTimeout: ReturnType<typeof setTimeout> | null = null

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (entry) {
        const { width, height } = entry.contentRect
        const w = Math.floor(width)
        const h = Math.floor(height)

        if (w === 0 || h === 0) return

        if (resizeTimeout) clearTimeout(resizeTimeout)
        resizeTimeout = setTimeout(() => {
          setSize(w, h)
          viewportApi.resize(w, h)
        }, 100) // Increased debounce for stability
      }
    })

    observer.observe(containerRef.current)
    return () => {
      if (resizeTimeout) clearTimeout(resizeTimeout)
      observer.disconnect()
    }
  }, [setSize])

  return (
    <div
      ref={containerRef}
      role="application"
      aria-label="3D Viewport"
      className="absolute inset-0 flex items-center justify-center cursor-crosshair touch-none"
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerLeave={handlePointerUp}
      onWheel={handleWheel}
      onContextMenu={handleContextMenu}
    >
      {isInitialized ? (
        <canvas
          ref={canvasRef}
          className="absolute inset-0 w-full h-full"
          style={{ imageRendering: "pixelated" }}
        />
      ) : (
        <div className="text-center pointer-events-none">
          <div className="mb-4 opacity-10">
            <HugeiconsIcon icon={CubeIcon} size={80} />
          </div>
          {error ? (
            <>
              <p className="text-sm text-red-400/80">{error}</p>
              <p className="mt-1 text-xs text-muted-foreground/40">wgpu initialization failed</p>
            </>
          ) : (
            <>
              <p className="text-sm text-muted-foreground/60">Initializing wgpu viewport...</p>
              <p className="mt-1 text-xs text-muted-foreground/40">
                Orbit: Alt+Click | Pan: Alt+Shift+Click | Zoom: Scroll
              </p>
            </>
          )}
        </div>
      )}
    </div>
  )
}

// ============================================================================
// MAIN VIEWPORT PANEL
// ============================================================================

export function ViewportPanel() {
  return (
    <div className="relative h-full w-full bg-[#1a1a1a] overflow-hidden">
      <ViewportCanvas />
      <ViewportToolbar />
    </div>
  )
}
