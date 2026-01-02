import {
  Button,
  cn,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
  Toaster,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@cadhy/ui"
import {
  Add01Icon,
  FolderOpenIcon,
  Home01Icon,
  Moon02Icon,
  Settings01Icon,
  SidebarLeft01Icon,
  SidebarRight01Icon,
  Sun01Icon,
} from "@hugeicons/core-free-icons"
import { HugeiconsIcon } from "@hugeicons/react"
import { useEffect, useState } from "react"
import { useLayoutStore } from "@/stores/layout-store"
import { useViewportStore } from "@/stores/viewport-store"
import { ViewportPanel } from "../viewport/ViewportPanel"
import { SettingsDialog } from "../dialogs/SettingsDialog"

export function AppLayout() {
  const { panels, togglePanel, theme, setTheme, borderRadius } = useLayoutStore()
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [isMacOS, setIsMacOS] = useState(false)

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
    <TooltipProvider delayDuration={300}>
      <div className="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground">
        {/* Titlebar */}
        <Titlebar
          isMacOS={isMacOS}
          panels={panels}
          togglePanel={togglePanel}
          theme={theme}
          setTheme={setTheme}
          onOpenSettings={() => setSettingsOpen(true)}
        />

        {/* Main content with panels */}
        <div className="flex-1 min-h-0">
          <ResizablePanelGroup direction="horizontal" className="h-full">
            {/* Left Panel */}
            {panels.left && (
              <>
                <ResizablePanel
                  id="left-panel"
                  order={1}
                  defaultSize={20}
                  minSize={15}
                  maxSize={35}
                  className="bg-card"
                >
                  <LeftPanel />
                </ResizablePanel>
                <ResizableHandle withHandle />
              </>
            )}

            {/* Center - Viewport */}
            <ResizablePanel id="viewport" order={2} defaultSize={60} minSize={40}>
              <ViewportPanel />
            </ResizablePanel>

            {/* Right Panel */}
            {panels.right && (
              <>
                <ResizableHandle withHandle />
                <ResizablePanel
                  id="right-panel"
                  order={3}
                  defaultSize={20}
                  minSize={15}
                  maxSize={35}
                  className="bg-card"
                >
                  <RightPanel />
                </ResizablePanel>
              </>
            )}
          </ResizablePanelGroup>
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
  panels: { left: boolean; right: boolean }
  togglePanel: (panel: "left" | "right") => void
  theme: "light" | "dark" | "system"
  setTheme: (theme: "light" | "dark" | "system") => void
  onOpenSettings: () => void
}

function Titlebar({ isMacOS, panels, togglePanel, theme, setTheme, onOpenSettings }: TitlebarProps) {
  return (
    <header
      data-tauri-drag-region
      className={cn(
        "relative flex shrink-0 items-center bg-background/80 backdrop-blur-sm border-b border-border/50",
        isMacOS ? "h-10 pl-[80px] pr-3" : "h-9 pl-3 pr-3"
      )}
    >
      {/* Left section */}
      <div className="flex items-center gap-2 z-10" data-tauri-drag-region>
        <LogoDropdown onOpenSettings={onOpenSettings} />

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant={panels.left ? "secondary" : "ghost"}
              size="icon"
              className="size-7"
              onClick={() => togglePanel("left")}
              data-tauri-drag-region="false"
            >
              <HugeiconsIcon icon={SidebarLeft01Icon} size={16} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Toggle Left Panel</TooltipContent>
        </Tooltip>
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

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant={panels.right ? "secondary" : "ghost"}
              size="icon"
              className="size-7"
              onClick={() => togglePanel("right")}
              data-tauri-drag-region="false"
            >
              <HugeiconsIcon icon={SidebarRight01Icon} size={16} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Toggle Right Panel</TooltipContent>
        </Tooltip>
      </div>
    </header>
  )
}

// ============================================================================
// LOGO DROPDOWN
// ============================================================================

function LogoDropdown({ onOpenSettings }: { onOpenSettings: () => void }) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="sm" className="h-7 gap-2 px-2" data-tauri-drag-region="false">
          <div className="size-5 rounded bg-primary flex items-center justify-center">
            <span className="text-[10px] font-bold text-primary-foreground">C</span>
          </div>
          <span className="text-xs font-medium">CADHY</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-48">
        <DropdownMenuItem>
          <HugeiconsIcon icon={Add01Icon} size={16} className="mr-2" />
          New Project
        </DropdownMenuItem>
        <DropdownMenuItem>
          <HugeiconsIcon icon={FolderOpenIcon} size={16} className="mr-2" />
          Open Project
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem>
          <HugeiconsIcon icon={Home01Icon} size={16} className="mr-2" />
          Home
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={onOpenSettings}>
          <HugeiconsIcon icon={Settings01Icon} size={16} className="mr-2" />
          Settings
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

// ============================================================================
// PANELS
// ============================================================================

function LeftPanel() {
  return (
    <div className="flex flex-col h-full">
      <div className="panel-header">
        <span className="panel-title">SCENE</span>
      </div>
      <div className="flex-1 p-3 text-sm text-muted-foreground">
        <p>No objects in scene</p>
      </div>
    </div>
  )
}

function RightPanel() {
  return (
    <div className="flex flex-col h-full">
      <div className="panel-header">
        <span className="panel-title">PROPERTIES</span>
      </div>
      <div className="flex-1 p-3 text-sm text-muted-foreground">
        <p>Select an object to view properties</p>
      </div>
    </div>
  )
}

// ============================================================================
// STATUS BAR
// ============================================================================

function StatusBar() {
  const { width, height, cursorPosition, fps, frameTime, viewMode } = useViewportStore()

  return (
    <footer className="h-6 flex items-center justify-between px-3 bg-muted/50 border-t border-border/50 text-[11px] font-mono text-muted-foreground">
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
