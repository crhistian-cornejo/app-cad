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
  { id: "performance", icon: ActivityIcon, label: "Performance" },
  { id: "language", icon: Globe02Icon, label: "Language" },
  { id: "appearance", icon: BrushIcon, label: "Appearance" },
  { id: "backup", icon: Files01Icon, label: "Backup" },
  { id: "server", icon: CloudServerIcon, label: "Server" },
  { id: "web", icon: WebProgrammingIcon, label: "Web publishing" },
  { id: "spacemouse", icon: SquareIcon, label: "SpaceMouse" },
  { id: "general", icon: InformationCircleIcon, label: "General" },
  { id: "advanced", icon: Alert01Icon, label: "Advanced" },
]

// ============================================================================
// NAVIGATION TAB
// ============================================================================

function NavigationTab() {
  const [preset, setPreset] = useState("maya")

  return (
    <div className="space-y-10">
      {/* Settings Grid */}
      <div className="space-y-6">
        <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50">
          Navigation Settings
        </h4>
        <div className="grid grid-cols-2 gap-x-10 gap-y-4">
          <CheckboxRow label="Zoom to cursor" defaultChecked />
          <CheckboxRow label="Zoom vertical" />
          <CheckboxRow label="Rotate around cursor" defaultChecked />
          <CheckboxRow label="Invert zoom direction" />
          <CheckboxRow label="Invert wheel direction" />
        </div>
      </div>

      {/* Presets List */}
      <div className="space-y-6">
        <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50">
          Navigation Preset
        </h4>
        <div className="space-y-1">
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
    <div className="flex items-center gap-3 group cursor-pointer">
      <Checkbox defaultChecked={defaultChecked} id={label} />
      <Label
        htmlFor={label}
        className="text-sm font-medium text-foreground/80 group-hover:text-foreground cursor-pointer transition-colors"
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
        "w-full text-left px-4 py-3 rounded-xl transition-all duration-200 group relative",
        isActive ? "bg-accent/50 shadow-sm" : "hover:bg-accent/30",
      )}
    >
      <div className="flex items-center justify-between">
        <div className="space-y-0.5">
          <div className="text-sm font-bold tracking-tight">{title}</div>
          <div className="text-xs text-muted-foreground font-medium">{description}</div>
        </div>
        {isActive && (
          <div className="size-5 flex items-center justify-center text-primary animate-in fade-in zoom-in duration-300">
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="3"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-label="Active"
            >
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
    <div className="space-y-10">
      {/* Theme */}
      <div className="space-y-6">
        <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50">
          Theme
        </h4>
        <div className="grid grid-cols-2 gap-4">
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
      <div className="space-y-6">
        <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50">
          Border Radius
        </h4>
        <div className="space-y-6 bg-accent/20 p-5 rounded-2xl border border-border/50">
          <div className="flex items-center justify-between px-2">
            <span className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
              Sharp
            </span>
            <div className="bg-primary/10 text-primary px-3 py-1 rounded-full text-xs font-bold font-mono">
              {borderRadius}px
            </div>
            <span className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
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
          <div className="grid grid-cols-3 gap-3">
            <div
              className="h-10 bg-primary text-primary-foreground flex items-center justify-center text-[10px] font-bold uppercase tracking-widest shadow-sm"
              style={{ borderRadius: `${borderRadius}px` }}
            >
              Button
            </div>
            <div
              className="h-10 bg-card border border-border flex items-center justify-center text-[10px] font-bold uppercase tracking-widest"
              style={{ borderRadius: `${borderRadius}px` }}
            >
              Card
            </div>
            <div
              className="h-10 bg-input flex items-center justify-center text-[10px] font-bold uppercase tracking-widest"
              style={{ borderRadius: `${borderRadius}px` }}
            >
              Input
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
        "flex-1 flex items-center justify-center gap-3 p-4 rounded-xl border transition-all duration-200",
        isActive
          ? "border-primary bg-primary/5 shadow-sm text-primary"
          : "border-border hover:border-primary/50 hover:bg-accent/50 text-muted-foreground hover:text-foreground",
      )}
    >
      <HugeiconsIcon icon={icon} className="size-5" />
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
    <div className="space-y-1">
      <SettingRow
        label="Show Grid"
        description="Display the grid in the 3D viewport"
        checked={showGrid}
        onCheckedChange={handleShowGridChange}
      />
      <SettingRow
        label="Show Axes"
        description="Display X, Y, Z axes indicator (coming soon)"
        checked={showAxes}
        onCheckedChange={handleShowAxesChange}
      />
      <SettingRow
        label="Anti-Aliasing"
        description="Smooth edges in the viewport (coming soon)"
        checked={antiAliasing}
        onCheckedChange={handleAntiAliasingChange}
      />
    </div>
  )
}

function SettingRow({
  label,
  description,
  checked,
  onCheckedChange,
}: {
  label: string
  description: string
  checked: boolean
  onCheckedChange: (checked: boolean) => void
}) {
  return (
    <div className="flex items-center justify-between p-4 rounded-xl hover:bg-accent/30 transition-colors group">
      <div className="space-y-0.5">
        <Label className="text-sm font-bold tracking-tight group-hover:text-foreground transition-colors cursor-pointer">
          {label}
        </Label>
        <p className="text-xs text-muted-foreground font-medium">{description}</p>
      </div>
      <Switch checked={checked} onCheckedChange={onCheckedChange} />
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
        className="w-[920px] max-w-[96vw] h-[640px] max-h-[85vh] p-0 flex flex-col overflow-hidden rounded-3xl border-border/40 shadow-2xl backdrop-blur-xl"
      >
        {/* Header */}
        <div className="shrink-0 px-8 h-16 border-b border-border/40 flex items-center justify-between bg-card/30">
          <h2 className="text-sm font-bold tracking-tight">Preferences</h2>
          <Button
            variant="ghost"
            size="icon"
            className="size-8 rounded-full hover:bg-accent/80"
            onClick={() => onOpenChange(false)}
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-label="Close"
            >
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </Button>
        </div>

        {/* Content */}
        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <aside className="w-56 p-4 shrink-0 bg-accent/10 border-r border-border/10">
            <div className="space-y-0.5">
              {SIDEBAR_ITEMS.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => setActiveSection(item.id)}
                  className={cn(
                    "w-full flex items-center gap-3 px-3 py-2 rounded-xl text-[13px] transition-all duration-200 group font-medium",
                    activeSection === item.id
                      ? "bg-accent text-foreground shadow-sm"
                      : "text-muted-foreground hover:bg-accent/50 hover:text-foreground",
                  )}
                >
                  <HugeiconsIcon
                    icon={item.icon}
                    className={cn(
                      "size-4 transition-colors",
                      activeSection === item.id ? "text-primary" : "group-hover:text-foreground",
                    )}
                  />
                  {item.label}
                </button>
              ))}
            </div>
          </aside>

          {/* Main Content */}
          <main className="flex-1 overflow-hidden min-w-0 bg-background/50">
            <ScrollArea className="h-full">
              <div className="px-12 py-10 w-full max-w-4xl">
                <h3 className="text-lg font-bold tracking-tight mb-10">
                  {SIDEBAR_ITEMS.find((item) => item.id === activeSection)?.label}
                </h3>

                <div className="animate-in fade-in slide-in-from-bottom-2 duration-300">
                  {activeSection === "viewport" && <ViewportTab />}
                  {activeSection === "navigation" && <NavigationTab />}
                  {activeSection === "appearance" && <AppearanceTab />}

                  {/* Placeholder for other sections */}
                  {!["viewport", "navigation", "appearance"].includes(activeSection) && (
                    <div className="py-24 flex flex-col items-center justify-center text-center space-y-5">
                      <div className="size-14 rounded-2xl bg-accent/20 flex items-center justify-center">
                        <HugeiconsIcon
                          icon={SIDEBAR_ITEMS.find((item) => item.id === activeSection)!.icon}
                          className="size-6 text-muted-foreground/30"
                        />
                      </div>
                      <div className="space-y-1">
                        <h4 className="font-bold text-sm uppercase tracking-wider text-muted-foreground">
                          {SIDEBAR_ITEMS.find((item) => item.id === activeSection)?.label}
                        </h4>
                        <p className="text-xs text-muted-foreground/60 font-medium">
                          This feature is currently under development.
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
