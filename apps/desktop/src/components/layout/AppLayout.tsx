import {
  Button,
  cn,
  CollapsiblePanelLayout,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
  DropdownMenuSub,
  DropdownMenuSubTrigger,
  DropdownMenuSubContent,
  DropdownMenuPortal,
  Toaster,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
  type PanelTab,
} from "@cadhy/ui"
import logoUrl from "@/assets/logo.png"
import {
  Add01Icon,
  ArrowDown01Icon,
  ArrowTurnBackwardIcon,
  ArrowTurnForwardIcon,
  Copy01Icon,
  Delete02Icon,
  FloppyDiskIcon,
  FolderOpenIcon,
  Moon02Icon,
  Scissor01Icon,
  Settings01Icon,
  Sun01Icon,
  ClipboardIcon,
  SquareIcon,
  GridIcon,
  Coordinate01Icon,
  ViewIcon,
  ArrowRight01Icon,
  ArrowUp01Icon,
  ArrowDown02Icon,
  ArrowLeft01Icon,
  LeftToRightBlockQuoteIcon,
  RightToLeftBlockQuoteIcon,
  GridViewIcon,
} from "@hugeicons/core-free-icons"
import { HugeiconsIcon } from "@hugeicons/react"
import { useEffect, useState, useCallback } from "react"
import { useLayoutStore } from "@/stores/layout-store"
import { useViewportStore } from "@/stores/viewport-store"
import { ViewportPanel } from "../viewport/ViewportPanel"
import { SettingsDialog } from "../dialogs/SettingsDialog"
import { ScenePanel } from "../panels/ScenePanel"
import { PropertiesPanel } from "../panels/PropertiesPanel"
import { projectApi } from "@/lib/tauri"
import { open as openDialog } from "@tauri-apps/plugin-dialog"
import { toast } from "sonner"

export function AppLayout() {
  const { theme, setTheme, borderRadius } = useLayoutStore()
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [isMacOS, setIsMacOS] = useState(false)

  // Panel tab definitions
  const leftPanelTabs: PanelTab[] = [
    {
      id: "scene",
      label: "Scene",
      content: <ScenePanel />,
    },
    {
      id: "layers",
      label: "Layers",
      badge: "WIP",
      content: (
        <div className="p-3 text-sm text-muted-foreground">
          <p>No layers</p>
        </div>
      ),
    },
  ]

  const rightPanelTabs: PanelTab[] = [
    {
      id: "properties",
      label: "Properties",
      content: <PropertiesPanel />,
    },
    {
      id: "inspector",
      label: "Inspector",
      badge: "WIP",
      content: (
        <div className="p-3 text-sm text-muted-foreground">
          <p>Inspector panel</p>
        </div>
      ),
    },
  ]

  // Detect platform
  useEffect(() => {
    setIsMacOS(navigator.platform.toLowerCase().includes("mac"))
  }, [])

  // Apply theme
  useEffect(() => {
    const root = document.documentElement
    if (theme === "dark") {
      root.classList.add("dark")
    } else {
      root.classList.remove("dark")
    }
  }, [theme])

  // Apply border radius CSS variable
  useEffect(() => {
    document.documentElement.style.setProperty("--radius", `${borderRadius}px`)
  }, [borderRadius])

  return (
    <TooltipProvider delay={300}>
      {/* Root is transparent to let wgpu show through viewport panel */}
      <div className="flex h-screen w-screen flex-col overflow-hidden text-foreground">
        {/* Titlebar */}
        <Titlebar
          isMacOS={isMacOS}
          theme={theme}
          setTheme={setTheme}
          onOpenSettings={() => setSettingsOpen(true)}
        />

        {/* Main content with panels */}
        <div className="flex-1 min-h-0 relative">
          <CollapsiblePanelLayout
            leftPanel={{
              tabs: leftPanelTabs,
              defaultCollapsed: false,
            }}
            rightPanel={{
              tabs: rightPanelTabs,
              defaultCollapsed: false,
            }}
          >
            <ViewportPanel />
          </CollapsiblePanelLayout>

          {/* Floating View Toolbar */}
          <ViewToolbar />
        </div>

        {/* Status Bar */}
        <StatusBar />
      </div>

      <Toaster position="bottom-right" />
      <SettingsDialog open={settingsOpen} onOpenChange={setSettingsOpen} />
    </TooltipProvider>
  )
}

// ============================================================================
// TITLEBAR
// ============================================================================

interface TitlebarProps {
  isMacOS: boolean
  theme: "light" | "dark" | "system"
  setTheme: (theme: "light" | "dark" | "system") => void
  onOpenSettings: () => void
}

function Titlebar({ isMacOS, theme, setTheme, onOpenSettings }: TitlebarProps) {
  return (
    <header
      data-tauri-drag-region
      className={cn(
        "relative flex shrink-0 items-center bg-background border-b border-border",
        isMacOS ? "h-10 pl-[76px] pr-3" : "h-9 pl-3 pr-3",
      )}
    >
      {/* Left section - Logo Menu Only */}
      <div className="flex items-center z-10" data-tauri-drag-region>
        <LogoDropdown onOpenSettings={onOpenSettings} isMacOS={isMacOS} />
      </div>

      {/* Center - App name */}
      <div
        className="absolute inset-0 flex items-center justify-center pointer-events-none"
        data-tauri-drag-region
      >
        <span className="text-xs font-medium text-muted-foreground">CADHY</span>
      </div>

      {/* Right section */}
      <div className="flex items-center gap-2 z-10 ml-auto" data-tauri-drag-region>
        {/* Theme toggle */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="size-7"
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
              data-tauri-drag-region="false"
            >
              <HugeiconsIcon icon={theme === "dark" ? Sun01Icon : Moon02Icon} size={16} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Toggle Theme</TooltipContent>
        </Tooltip>
      </div>
    </header>
  )
}

// ============================================================================
// LOGO DROPDOWN - Main App Menu
// ============================================================================

interface LogoDropdownProps {
  onOpenSettings: () => void
  isMacOS: boolean
}

function LogoDropdown({ onOpenSettings, isMacOS }: LogoDropdownProps) {
  const [isOpen, setIsOpen] = useState(false)
  const { showGrid, showAxes, setShowGrid, setShowAxes } = useLayoutStore()
  const { projection, setProjection } = useViewportStore()

  const cmdKey = isMacOS ? "⌘" : "Ctrl+"

  // File actions
  const handleNewProject = useCallback(async () => {
    try {
      await projectApi.newProject()
      toast.success("Created a new empty project")
    } catch (e) {
      console.error("Failed to create new project:", e)
      toast.error("Failed to create new project")
    }
  }, [])

  const handleOpenProject = useCallback(async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [
          { name: "CADHY Project", extensions: ["cadhy", "cad"] },
          { name: "STEP Files", extensions: ["step", "stp"] },
          { name: "All Files", extensions: ["*"] },
        ],
      })
      if (selected) {
        toast.info(`Selected: ${selected}`)
      }
    } catch (e) {
      console.error("Failed to open project:", e)
    }
  }, [])

  const handleSave = useCallback(() => {
    toast.info("Save functionality coming soon")
  }, [])

  // Edit actions
  const handleUndo = useCallback(async () => {
    try {
      await projectApi.undo()
    } catch (e) {
      console.error("Failed to undo:", e)
    }
  }, [])

  const handleRedo = useCallback(async () => {
    try {
      await projectApi.redo()
    } catch (e) {
      console.error("Failed to redo:", e)
    }
  }, [])

  const handleCut = useCallback(() => toast.info("Cut coming soon"), [])
  const handleCopy = useCallback(() => toast.info("Copy coming soon"), [])
  const handlePaste = useCallback(() => toast.info("Paste coming soon"), [])
  const handleDelete = useCallback(() => toast.info("Delete coming soon"), [])

  // View actions
  const handleToggleGrid = useCallback(() => setShowGrid(!showGrid), [showGrid, setShowGrid])
  const handleToggleAxes = useCallback(() => setShowAxes(!showAxes), [showAxes, setShowAxes])
  const handleToggleProjection = useCallback(() => {
    setProjection(projection === "perspective" ? "orthographic" : "perspective")
  }, [projection, setProjection])

  return (
    <DropdownMenu onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="h-7 gap-1.5 px-1.5 pr-2 rounded-lg"
          data-tauri-drag-region="false"
        >
          <img src={logoUrl} alt="CADHY" className="size-5 rounded shadow-sm" />
          <span className="text-xs font-semibold tracking-tight text-foreground/90">CADHY</span>
          <div
            className={cn("transition-transform duration-200", isOpen ? "rotate-180" : "rotate-0")}
          >
            <HugeiconsIcon
              icon={ArrowDown01Icon}
              size={10}
              className="text-muted-foreground/50 ml-0.5"
            />
          </div>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-56">
        {/* === FILE SUBMENU === */}
        <DropdownMenuSub>
          <DropdownMenuSubTrigger>
            <HugeiconsIcon icon={FolderOpenIcon} size={16} className="mr-2" />
            File
          </DropdownMenuSubTrigger>
          <DropdownMenuPortal>
            <DropdownMenuSubContent className="w-52">
              <DropdownMenuItem onClick={handleNewProject}>
                <HugeiconsIcon icon={Add01Icon} size={16} className="mr-2" />
                New Project
                <DropdownMenuShortcut>{cmdKey}N</DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuItem onClick={handleOpenProject}>
                <HugeiconsIcon icon={FolderOpenIcon} size={16} className="mr-2" />
                Open...
                <DropdownMenuShortcut>{cmdKey}O</DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={handleSave}>
                <HugeiconsIcon icon={FloppyDiskIcon} size={16} className="mr-2" />
                Save
                <DropdownMenuShortcut>{cmdKey}S</DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuItem onClick={handleSave}>
                <HugeiconsIcon icon={FloppyDiskIcon} size={16} className="mr-2" />
                Save As...
                <DropdownMenuShortcut>{cmdKey}⇧S</DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem>
                Export...
                <DropdownMenuShortcut>{cmdKey}E</DropdownMenuShortcut>
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuPortal>
        </DropdownMenuSub>

        {/* === EDIT SUBMENU === */}
        <DropdownMenuSub>
          <DropdownMenuSubTrigger>
            <HugeiconsIcon icon={Scissor01Icon} size={16} className="mr-2" />
            Edit
          </DropdownMenuSubTrigger>
          <DropdownMenuPortal>
            <DropdownMenuSubContent className="w-52">
              <DropdownMenuItem onClick={handleUndo}>
                <HugeiconsIcon icon={ArrowTurnBackwardIcon} size={16} className="mr-2" />
                Undo
                <DropdownMenuShortcut>{cmdKey}Z</DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuItem onClick={handleRedo}>
                <HugeiconsIcon icon={ArrowTurnForwardIcon} size={16} className="mr-2" />
                Redo
                <DropdownMenuShortcut>
                  {cmdKey}
                  {isMacOS ? "⇧Z" : "Y"}
                </DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={handleCut}>
                <HugeiconsIcon icon={Scissor01Icon} size={16} className="mr-2" />
                Cut
                <DropdownMenuShortcut>{cmdKey}X</DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuItem onClick={handleCopy}>
                <HugeiconsIcon icon={Copy01Icon} size={16} className="mr-2" />
                Copy
                <DropdownMenuShortcut>{cmdKey}C</DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuItem onClick={handlePaste}>
                <HugeiconsIcon icon={ClipboardIcon} size={16} className="mr-2" />
                Paste
                <DropdownMenuShortcut>{cmdKey}V</DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={handleDelete} className="text-destructive">
                <HugeiconsIcon icon={Delete02Icon} size={16} className="mr-2" />
                Delete
                <DropdownMenuShortcut>Del</DropdownMenuShortcut>
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuPortal>
        </DropdownMenuSub>

        {/* === VIEW SUBMENU === */}
        <DropdownMenuSub>
          <DropdownMenuSubTrigger>
            <HugeiconsIcon icon={ViewIcon} size={16} className="mr-2" />
            View
          </DropdownMenuSubTrigger>
          <DropdownMenuPortal>
            <DropdownMenuSubContent className="w-52">
              <DropdownMenuItem onClick={handleToggleProjection}>
                <HugeiconsIcon
                  icon={projection === "perspective" ? SquareIcon : ViewIcon}
                  size={16}
                  className="mr-2"
                />
                {projection === "perspective" ? "Orthographic" : "Perspective"}
                <DropdownMenuShortcut>5</DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={handleToggleGrid}>
                <HugeiconsIcon icon={GridIcon} size={16} className="mr-2" />
                {showGrid ? "Hide Grid" : "Show Grid"}
                <DropdownMenuShortcut>G</DropdownMenuShortcut>
              </DropdownMenuItem>
              <DropdownMenuItem onClick={handleToggleAxes}>
                <HugeiconsIcon icon={Coordinate01Icon} size={16} className="mr-2" />
                {showAxes ? "Hide Axes" : "Show Axes"}
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuPortal>
        </DropdownMenuSub>

        <DropdownMenuSeparator />

        {/* === SETTINGS === */}
        <DropdownMenuItem onClick={onOpenSettings}>
          <HugeiconsIcon icon={Settings01Icon} size={16} className="mr-2" />
          Settings
          <DropdownMenuShortcut>{cmdKey},</DropdownMenuShortcut>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

// ============================================================================
// VIEW TOOLBAR - Floating toolbar for camera views
// ============================================================================

function ViewToolbar() {
  const { setView } = useViewportStore()

  const handleSetView = useCallback(
    (viewId: "front" | "back" | "right" | "left" | "top" | "bottom") => {
      setView(viewId)
      toast.success(`${viewId.charAt(0).toUpperCase() + viewId.slice(1)} view`)
    },
    [setView],
  )

  return (
    <div className="absolute top-3 right-3 z-20">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="secondary"
            size="icon"
            className="size-8 shadow-md bg-background/80 backdrop-blur-sm border border-border"
          >
            <HugeiconsIcon icon={GridViewIcon} size={16} />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-44">
          <DropdownMenuItem onClick={() => handleSetView("front")}>
            <HugeiconsIcon icon={ArrowUp01Icon} size={16} className="mr-2" />
            Front
            <DropdownMenuShortcut>1</DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => handleSetView("back")}>
            <HugeiconsIcon icon={ArrowDown02Icon} size={16} className="mr-2" />
            Back
            <DropdownMenuShortcut>Ctrl+1</DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={() => handleSetView("right")}>
            <HugeiconsIcon icon={ArrowRight01Icon} size={16} className="mr-2" />
            Right
            <DropdownMenuShortcut>3</DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => handleSetView("left")}>
            <HugeiconsIcon icon={ArrowLeft01Icon} size={16} className="mr-2" />
            Left
            <DropdownMenuShortcut>Ctrl+3</DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={() => handleSetView("top")}>
            <HugeiconsIcon icon={LeftToRightBlockQuoteIcon} size={16} className="mr-2 rotate-90" />
            Top
            <DropdownMenuShortcut>7</DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => handleSetView("bottom")}>
            <HugeiconsIcon icon={RightToLeftBlockQuoteIcon} size={16} className="mr-2 rotate-90" />
            Bottom
            <DropdownMenuShortcut>Ctrl+7</DropdownMenuShortcut>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}

// ============================================================================
// STATUS BAR
// ============================================================================

function StatusBar() {
  const { width, height, cursorPosition, fps, frameTime, viewMode, startFpsPolling } =
    useViewportStore()

  // Start FPS polling when component mounts
  useEffect(() => {
    const stopPolling = startFpsPolling()
    return stopPolling
  }, [startFpsPolling])

  return (
    <footer className="h-6 flex items-center justify-between px-3 bg-background border-t border-border text-[11px] font-mono text-muted-foreground">
      {/* Left section - Status */}
      <div className="flex items-center gap-4">
        <span className="text-green-500">Ready</span>
        <span className="text-muted-foreground/60">|</span>
        <span>{viewMode === "solid" ? "Solid" : "Wireframe"}</span>
      </div>

      {/* Center section - Coordinates */}
      <div className="flex items-center gap-3">
        <span>
          X: <span className="text-foreground">{cursorPosition.x.toFixed(2)}</span>
        </span>
        <span>
          Y: <span className="text-foreground">{cursorPosition.y.toFixed(2)}</span>
        </span>
        <span>
          Z: <span className="text-foreground">{cursorPosition.z.toFixed(2)}</span>
        </span>
      </div>

      {/* Right section - Performance */}
      <div className="flex items-center gap-4">
        <span>
          {width} x {height}
        </span>
        <span className="text-muted-foreground/60">|</span>
        <span
          className={cn(
            fps >= 30 ? "text-green-500" : fps >= 15 ? "text-yellow-500" : "text-red-500",
          )}
        >
          {fps} FPS
        </span>
        <span className="text-muted-foreground/60">({frameTime.toFixed(1)}ms)</span>
      </div>
    </footer>
  )
}
