/**
 * Settings Dialog - CADHY
 *
 * Complete settings dialog with appearance, viewport, and general options.
 */

import {
  Button,
  cn,
  Dialog,
  DialogContent,
  Label,
  ScrollArea,
  Slider,
  Switch,
  Checkbox,
} from "@cadhy/ui"
import {
  BrushIcon,
  GridIcon,
  Moon02Icon,
  Sun01Icon,
  MouseIcon,
  ActivityIcon,
  Globe02Icon,
  Files01Icon,
  CloudServerIcon,
  WebProgrammingIcon,
  SquareIcon,
  InformationCircleIcon,
  Alert01Icon,
  Settings01Icon,
} from "@hugeicons/core-free-icons"
import { HugeiconsIcon } from "@hugeicons/react"
import { useState } from "react"
import { useLayoutStore } from "@/stores/layout-store"
import { viewportApi } from "@/lib/tauri"

// ============================================================================
// TYPES
// ============================================================================

interface SettingsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

type SettingsSection =
  | "viewport"
  | "navigation"
  | "performance"
  | "language"
  | "appearance"
  | "backup"
  | "server"
  | "web"
  | "spacemouse"
  | "general"
  | "advanced"

interface SidebarItem {
  id: SettingsSection
  icon: any
  label: string
}

// ============================================================================
// SIDEBAR ITEMS
// ============================================================================

const SIDEBAR_ITEMS: SidebarItem[] = [
  { id: "viewport", icon: GridIcon, label: "Viewport" },
  { id: "navigation", icon: MouseIcon, label: "Navigation" },
  { id: "spacemouse", icon: SquareIcon, label: "SpaceMouse" },
  { id: "performance", icon: ActivityIcon, label: "Performance" },
  { id: "appearance", icon: BrushIcon, label: "Appearance" },
  { id: "language", icon: Globe02Icon, label: "Language" },
  { id: "backup", icon: Files01Icon, label: "Backup" },
  { id: "server", icon: CloudServerIcon, label: "Server" },
  { id: "web", icon: WebProgrammingIcon, label: "Web publishing" },
  { id: "general", icon: InformationCircleIcon, label: "General" },
  { id: "advanced", icon: Alert01Icon, label: "Advanced" },
]

// ============================================================================
// NAVIGATION TAB
// ============================================================================

function NavigationTab() {
  const [preset, setPreset] = useState("maya")

  return (
    <div className="space-y-8">
      {/* Settings Grid */}
      <div className="space-y-4">
        <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/40">
          Navigation Settings
        </h4>
        <div className="grid grid-cols-2 gap-x-8 gap-y-1">
          <CheckboxRow label="Zoom to cursor" defaultChecked />
          <CheckboxRow label="Zoom vertical" />
          <CheckboxRow label="Rotate around cursor" defaultChecked />
          <CheckboxRow label="Invert zoom direction" />
          <CheckboxRow label="Invert wheel direction" />
        </div>
      </div>

      {/* Presets List */}
      <div className="space-y-4">
        <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/40">
          Navigation Preset
        </h4>
        <div className="grid grid-cols-2 gap-2">
          <PresetItem
            title="Plasticity default"
            description="Middle mouse to orbit, right mouse to pan."
            isActive={preset === "plasticity"}
            onClick={() => setPreset("plasticity")}
          />
          <PresetItem
            title="Touchpad & Magic Mouse"
            description="For laptop touchpads and Apple Magic Mouse."
            isActive={preset === "touchpad"}
            onClick={() => setPreset("touchpad")}
          />
          <PresetItem
            title="Blender"
            description="Middle mouse to orbit, shift-middle mouse to pan."
            isActive={preset === "blender"}
            onClick={() => setPreset("blender")}
          />
          <PresetItem
            title="Maya"
            description="Alt-left mouse to orbit, alt-middle mouse to pan."
            isActive={preset === "maya"}
            onClick={() => setPreset("maya")}
          />
          <PresetItem
            title="Mol3D"
            description="Right mouse to orbit, shift-right mouse to pan."
            isActive={preset === "mol3d"}
            onClick={() => setPreset("mol3d")}
          />
        </div>
      </div>
    </div>
  )
}

function CheckboxRow({ label, defaultChecked }: { label: string; defaultChecked?: boolean }) {
  return (
    <div className="flex items-center gap-2.5 py-1.5 px-2.5 rounded-lg hover:bg-accent/30 transition-colors group cursor-pointer">
      <Checkbox defaultChecked={defaultChecked} id={label} />
      <Label
        htmlFor={label}
        className="text-[13px] font-medium text-muted-foreground group-hover:text-foreground cursor-pointer transition-colors"
      >
        {label}
      </Label>
    </div>
  )
}

function PresetItem({
  title,
  description,
  isActive,
  onClick,
}: {
  title: string
  description: string
  isActive: boolean
  onClick: () => void
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "w-full text-left px-4 py-3 rounded-xl transition-all duration-200 group relative border",
        isActive
          ? "bg-accent/40 shadow-sm border-primary/20"
          : "bg-accent/5 hover:bg-accent/20 border-transparent",
      )}
    >
      <div className="flex items-center justify-between gap-3">
        <div className="space-y-0.5">
          <div
            className={cn(
              "text-[13px] font-bold tracking-tight transition-colors",
              isActive ? "text-foreground" : "text-muted-foreground group-hover:text-foreground",
            )}
          >
            {title}
          </div>
          <div className="text-[10px] text-muted-foreground/60 font-medium leading-tight">
            {description}
          </div>
        </div>
        {isActive && (
          <div className="shrink-0 size-4 flex items-center justify-center text-primary animate-in fade-in zoom-in duration-300">
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="3"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <title>Selected</title>
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </div>
        )}
      </div>
    </button>
  )
}

// ============================================================================
// APPEARANCE TAB
// ============================================================================

function AppearanceTab() {
  const { theme, setTheme, borderRadius, setBorderRadius } = useLayoutStore()

  return (
    <div className="space-y-8">
      {/* Theme */}
      <div className="space-y-4">
        <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/40">
          Theme
        </h4>
        <div className="grid grid-cols-2 gap-3">
          <ThemeButton
            icon={Sun01Icon}
            label="Light"
            isActive={theme === "light"}
            onClick={() => setTheme("light")}
          />
          <ThemeButton
            icon={Moon02Icon}
            label="Dark"
            isActive={theme === "dark"}
            onClick={() => setTheme("dark")}
          />
        </div>
      </div>

      {/* Border Radius */}
      <div className="space-y-4">
        <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/40">
          Border Radius
        </h4>
        <div className="space-y-6 bg-accent/10 p-6 rounded-2xl border border-border/10">
          <div className="flex items-center justify-between px-1">
            <span className="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/40">
              Sharp
            </span>
            <div className="bg-primary/10 text-primary px-3 py-1 rounded-full text-[10px] font-bold font-mono">
              {borderRadius}px
            </div>
            <span className="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/40">
              Round
            </span>
          </div>
          <Slider
            value={[borderRadius]}
            onValueChange={([value]) => setBorderRadius(value)}
            min={0}
            max={16}
            step={1}
            className="w-full"
          />
          {/* Preview */}
          <div className="grid grid-cols-3 gap-4 pt-2">
            <div className="space-y-1.5">
              <div
                className="h-10 bg-primary text-primary-foreground flex items-center justify-center text-[9px] font-bold uppercase tracking-widest shadow-lg shadow-primary/20"
                style={{ borderRadius: `${borderRadius}px` }}
              >
                Button
              </div>
              <p className="text-[8px] text-center font-bold text-muted-foreground/40 uppercase tracking-tighter">
                Primary
              </p>
            </div>
            <div className="space-y-1.5">
              <div
                className="h-10 bg-card border border-border/60 flex items-center justify-center text-[9px] font-bold uppercase tracking-widest"
                style={{ borderRadius: `${borderRadius}px` }}
              >
                Card
              </div>
              <p className="text-[8px] text-center font-bold text-muted-foreground/40 uppercase tracking-tighter">
                Surface
              </p>
            </div>
            <div className="space-y-1.5">
              <div
                className="h-10 bg-input border border-border/40 flex items-center justify-center text-[9px] font-bold uppercase tracking-widest"
                style={{ borderRadius: `${borderRadius}px` }}
              >
                Input
              </div>
              <p className="text-[8px] text-center font-bold text-muted-foreground/40 uppercase tracking-tighter">
                Control
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function ThemeButton({
  icon,
  label,
  isActive,
  onClick,
}: {
  icon: typeof Sun01Icon
  label: string
  isActive: boolean
  onClick: () => void
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "flex-1 flex items-center justify-center gap-2.5 p-4 rounded-xl border transition-all duration-200 group",
        isActive
          ? "border-primary bg-primary/5 shadow-sm text-primary"
          : "border-border/40 bg-accent/5 hover:bg-accent/20 text-muted-foreground hover:text-foreground",
      )}
    >
      <HugeiconsIcon
        icon={icon}
        className={cn(
          "size-5 transition-transform group-hover:scale-110",
          isActive && "animate-pulse",
        )}
      />
      <span className="text-xs font-bold uppercase tracking-widest">{label}</span>
    </button>
  )
}

// ============================================================================
// VIEWPORT TAB
// ============================================================================

function ViewportTab() {
  const { showGrid, setShowGrid, showAxes, setShowAxes, antiAliasing, setAntiAliasing } =
    useLayoutStore()

  // Sync settings with backend when they change
  const handleShowGridChange = async (checked: boolean) => {
    setShowGrid(checked)
    try {
      await viewportApi.setSettings({
        show_grid: checked,
        show_axes: showAxes,
        anti_aliasing: antiAliasing,
      })
    } catch (e) {
      console.error("Failed to update viewport settings:", e)
    }
  }

  const handleShowAxesChange = async (checked: boolean) => {
    setShowAxes(checked)
    try {
      await viewportApi.setSettings({
        show_grid: showGrid,
        show_axes: checked,
        anti_aliasing: antiAliasing,
      })
    } catch (e) {
      console.error("Failed to update viewport settings:", e)
    }
  }

  const handleAntiAliasingChange = async (checked: boolean) => {
    setAntiAliasing(checked)
    try {
      await viewportApi.setSettings({
        show_grid: showGrid,
        show_axes: showAxes,
        anti_aliasing: checked,
      })
    } catch (e) {
      console.error("Failed to update viewport settings:", e)
    }
  }

  return (
    <div className="space-y-8">
      <div className="space-y-4">
        <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/40">
          Viewport Display
        </h4>
        <div className="grid grid-cols-2 gap-3">
          <SettingRow
            label="Show Grid"
            description="Display the ground grid in the viewport"
            checked={showGrid}
            onCheckedChange={handleShowGridChange}
            icon={GridIcon}
          />
          <SettingRow
            label="Show Axes"
            description="Display X, Y, Z axes indicator"
            checked={showAxes}
            onCheckedChange={handleShowAxesChange}
            icon={ActivityIcon}
          />
        </div>
      </div>

      <div className="space-y-4">
        <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/40">
          Rendering Quality
        </h4>
        <div className="grid grid-cols-2 gap-3">
          <SettingRow
            label="Anti-Aliasing"
            description="Smooth jagged edges in the 3D scene"
            checked={antiAliasing}
            onCheckedChange={handleAntiAliasingChange}
            icon={BrushIcon}
          />
        </div>
      </div>
    </div>
  )
}

function SettingRow({
  label,
  description,
  checked,
  onCheckedChange,
  icon,
}: {
  label: string
  description: string
  checked: boolean
  onCheckedChange: (checked: boolean) => void
  icon: any
}) {
  return (
    <div
      className={cn(
        "flex items-center justify-between p-4 rounded-xl transition-all duration-200 group border",
        checked
          ? "bg-accent/40 shadow-sm border-primary/20"
          : "bg-accent/5 hover:bg-accent/15 border-transparent",
      )}
    >
      <div className="flex items-center gap-3.5 flex-1 pr-4">
        <div
          className={cn(
            "size-9 rounded-lg flex items-center justify-center transition-colors",
            checked ? "bg-primary/10 text-primary" : "bg-accent/10 text-muted-foreground/40",
          )}
        >
          <HugeiconsIcon icon={icon} className="size-4.5" />
        </div>
        <div className="space-y-0.5">
          <Label
            className={cn(
              "text-[13px] font-bold tracking-tight transition-colors cursor-pointer",
              checked ? "text-foreground" : "text-muted-foreground group-hover:text-foreground",
            )}
            onClick={() => onCheckedChange(!checked)}
          >
            {label}
          </Label>
          <p className="text-[10px] text-muted-foreground/60 font-medium leading-tight">
            {description}
          </p>
        </div>
      </div>
      <Switch size="sm" checked={checked} onCheckedChange={onCheckedChange} />
    </div>
  )
}

// ============================================================================
// MAIN DIALOG
// ============================================================================

export function SettingsDialog({ open, onOpenChange }: SettingsDialogProps) {
  const [activeSection, setActiveSection] = useState<SettingsSection>("viewport")

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        showCloseButton={false}
        size="5xl"
        className="h-[720px] max-h-[92vh] p-0 flex flex-col overflow-hidden rounded-[calc(var(--radius)*2.5)] border-border/20 shadow-[0_0_50px_-12px_rgba(0,0,0,0.5)] backdrop-blur-3xl"
      >
        {/* Header */}
        <div className="shrink-0 px-8 h-16 flex items-center justify-between bg-card/20">
          <div className="flex items-center gap-3">
            <div className="size-8 rounded-lg bg-primary/10 flex items-center justify-center">
              <HugeiconsIcon icon={Settings01Icon} className="size-4.5 text-primary" />
            </div>
            <h2 className="text-sm font-bold tracking-tight">Preferences</h2>
          </div>
          <Button
            variant="ghost"
            size="icon"
            className="size-8 rounded-xl hover:bg-accent/80 transition-all active:scale-90"
            onClick={() => onOpenChange(false)}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <title>Close Preferences</title>
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </Button>
        </div>

        {/* Content */}
        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <aside className="w-56 p-4 shrink-0 bg-accent/5 border-r border-border/5">
            <div className="space-y-1">
              {SIDEBAR_ITEMS.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => setActiveSection(item.id)}
                  className={cn(
                    "w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-[12.5px] transition-all duration-200 group font-bold",
                    activeSection === item.id
                      ? "bg-accent text-foreground shadow-sm"
                      : "text-muted-foreground/60 hover:bg-accent/40 hover:text-foreground",
                  )}
                >
                  <HugeiconsIcon
                    icon={item.icon}
                    className={cn(
                      "size-4.5 transition-all",
                      activeSection === item.id
                        ? "text-primary scale-110"
                        : "group-hover:text-foreground group-hover:scale-105",
                    )}
                  />
                  {item.label}
                </button>
              ))}
            </div>
          </aside>

          {/* Main Content */}
          <main className="flex-1 overflow-hidden min-w-0 bg-background/30">
            <ScrollArea className="h-full">
              <div className="px-10 py-10 w-full max-w-full">
                <div className="flex items-center gap-3 mb-10">
                  <HugeiconsIcon
                    icon={SIDEBAR_ITEMS.find((item) => item.id === activeSection)?.icon}
                    className="size-7 text-primary/80"
                  />
                  <h3 className="text-xl font-bold tracking-tight">
                    {SIDEBAR_ITEMS.find((item) => item.id === activeSection)?.label}
                  </h3>
                </div>

                <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
                  {activeSection === "viewport" && <ViewportTab />}
                  {activeSection === "navigation" && <NavigationTab />}
                  {activeSection === "appearance" && <AppearanceTab />}

                  {/* Placeholder for other sections */}
                  {!["viewport", "navigation", "appearance"].includes(activeSection) && (
                    <div className="py-24 flex flex-col items-center justify-center text-center space-y-8">
                      <div className="size-24 rounded-[2.5rem] bg-accent/10 flex items-center justify-center shadow-inner relative group/icon">
                        <div className="absolute inset-0 rounded-[2.5rem] bg-primary/5 animate-pulse" />
                        <HugeiconsIcon
                          icon={SIDEBAR_ITEMS.find((item) => item.id === activeSection)!.icon}
                          className="size-10 text-muted-foreground/10 group-hover/icon:text-primary/20 transition-colors duration-500"
                        />
                      </div>
                      <div className="space-y-3">
                        <h4 className="font-bold text-xl uppercase tracking-widest text-muted-foreground/30">
                          {SIDEBAR_ITEMS.find((item) => item.id === activeSection)?.label}
                        </h4>
                        <p className="text-sm text-muted-foreground/20 font-bold max-w-[320px] mx-auto leading-relaxed uppercase tracking-tighter">
                          Section under active development
                        </p>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </ScrollArea>
          </main>
        </div>
      </DialogContent>
    </Dialog>
  )
}
