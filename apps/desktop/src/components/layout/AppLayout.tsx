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
  FolderOpenIcon,
  Moon02Icon,
  Settings01Icon,
  Sun01Icon,
} from "@hugeicons/core-free-icons"
import { HugeiconsIcon } from "@hugeicons/react"
import { useEffect, useState, useCallback } from "react"
import { useLayoutStore } from "@/stores/layout-store"
import { useViewportStore } from "@/stores/viewport-store"
import { ViewportPanel } from "../viewport/ViewportPanel"
import { SettingsDialog } from "../dialogs/SettingsDialog"
import { projectApi } from "@/lib/tauri"
import { open as openDialog } from "@tauri-apps/plugin-dialog"
import { toast } from "sonner"
import { getCurrentWindow } from "@tauri-apps/api/window"

export function AppLayout() {
  const { theme, setTheme, borderRadius } = useLayoutStore()
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [isMacOS, setIsMacOS] = useState(false)

  // Panel tab definitions
  const leftPanelTabs: PanelTab[] = [
    {
      id: "scene",
      label: "Scene",
      badge: "WIP",
      content: (
        <div className="p-3 text-sm text-muted-foreground">
          <p>No objects in scene</p>
        </div>
      ),
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
      badge: "WIP",
      content: (
        <div className="p-3 text-sm text-muted-foreground">
          <p>Select an object to view properties</p>
        </div>
      ),
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
        <div className="flex-1 min-h-0">
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
        isMacOS ? "h-10 pl-[60px] pr-3" : "h-9 pl-3 pr-3"
      )}
    >
      {/* Left section */}
      <div className="flex items-center gap-2 z-10" data-tauri-drag-region>
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
              <HugeiconsIcon
                icon={theme === "dark" ? Sun01Icon : Moon02Icon}
                size={16}
              />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Toggle Theme</TooltipContent>
        </Tooltip>
      </div>
    </header>
  )
}

// ============================================================================
// LOGO DROPDOWN
// ============================================================================

interface LogoDropdownProps {
  onOpenSettings: () => void
  isMacOS: boolean
}

function LogoDropdown({ onOpenSettings, isMacOS }: LogoDropdownProps) {
  const [isOpen, setIsOpen] = useState(false)
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
          { name: "All Files", extensions: ["*"] },
        ],
      })

      if (selected) {
        // TODO: Implement project loading
        toast.info(`Selected: ${selected}`)
      }
    } catch (e) {
      console.error("Failed to open project:", e)
    }
  }, [])

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

  const cmdKey = isMacOS ? "⌘" : "Ctrl+"

  return (
    <DropdownMenu onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="sm" className="h-7 gap-1.5 px-1.5 pr-2 rounded-lg" data-tauri-drag-region="false">
          <img src={logoUrl} alt="CADHY" className="size-5 rounded shadow-sm" />
          <span className="text-xs font-semibold tracking-tight text-foreground/90">CADHY</span>
          <div className={cn("transition-transform duration-200", isOpen ? "rotate-180" : "rotate-0")}>
            <HugeiconsIcon icon={ArrowDown01Icon} size={10} className="text-muted-foreground/50 ml-0.5" />
          </div>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-52">
        <DropdownMenuItem onClick={handleNewProject}>
          <HugeiconsIcon icon={Add01Icon} size={16} className="mr-2" />
          New Project
          <DropdownMenuShortcut>{cmdKey}N</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={handleOpenProject}>
          <HugeiconsIcon icon={FolderOpenIcon} size={16} className="mr-2" />
          Open Project
          <DropdownMenuShortcut>{cmdKey}O</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={handleUndo}>
          <HugeiconsIcon icon={ArrowTurnBackwardIcon} size={16} className="mr-2" />
          Undo
          <DropdownMenuShortcut>{cmdKey}Z</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={handleRedo}>
          <HugeiconsIcon icon={ArrowTurnForwardIcon} size={16} className="mr-2" />
          Redo
          <DropdownMenuShortcut>{cmdKey}{isMacOS ? "⇧Z" : "Y"}</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
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
// STATUS BAR
// ============================================================================

function StatusBar() {
  const { width, height, cursorPosition, fps, frameTime, viewMode } = useViewportStore()

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
        <span>X: <span className="text-foreground">{cursorPosition.x.toFixed(2)}</span></span>
        <span>Y: <span className="text-foreground">{cursorPosition.y.toFixed(2)}</span></span>
        <span>Z: <span className="text-foreground">{cursorPosition.z.toFixed(2)}</span></span>
      </div>

      {/* Right section - Performance */}
      <div className="flex items-center gap-4">
        <span>{width} x {height}</span>
        <span className="text-muted-foreground/60">|</span>
        <span className={cn(
          fps >= 30 ? "text-green-500" : fps >= 15 ? "text-yellow-500" : "text-red-500"
        )}>
          {fps} FPS
        </span>
        <span className="text-muted-foreground/60">({frameTime.toFixed(1)}ms)</span>
      </div>
    </footer>
  )
}
