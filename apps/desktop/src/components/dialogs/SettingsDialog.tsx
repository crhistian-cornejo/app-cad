/**
 * Settings Dialog - CADHY
 *
 * Complete settings dialog with appearance, viewport, and general options.
 */

import { Button, cn, Dialog, DialogContent, Label, ScrollArea, Slider, Switch } from "@cadhy/ui"
import {
  BrushIcon,
  GridIcon,
  Moon02Icon,
  Settings01Icon,
  Sun01Icon,
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

type SettingsSection = "appearance" | "viewport"

interface SidebarItem {
  id: SettingsSection
  icon: typeof Settings01Icon
  label: string
}

// ============================================================================
// SIDEBAR ITEMS
// ============================================================================

const SIDEBAR_ITEMS: SidebarItem[] = [
  { id: "appearance", icon: BrushIcon, label: "Appearance" },
  { id: "viewport", icon: GridIcon, label: "Viewport" },
]

// ============================================================================
// APPEARANCE TAB
// ============================================================================

function AppearanceTab() {
  const { theme, setTheme, borderRadius, setBorderRadius } = useLayoutStore()

  return (
    <div className="space-y-8">
      {/* Theme */}
      <div className="space-y-4">
        <div>
          <h3 className="text-sm font-medium">Theme</h3>
          <p className="text-xs text-muted-foreground mt-1">Select your preferred color theme</p>
        </div>
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
      <div className="space-y-4">
        <div>
          <h3 className="text-sm font-medium">Border Radius</h3>
          <p className="text-xs text-muted-foreground mt-1">
            Adjust the roundness of UI elements (0px - 16px)
          </p>
        </div>
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">Sharp</span>
            <span className="text-sm font-mono font-medium">{borderRadius}px</span>
            <span className="text-xs text-muted-foreground">Round</span>
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
          <div className="grid grid-cols-3 gap-4 mt-6">
            <div
              className="flex-1 h-12 bg-primary/10 border border-border flex items-center justify-center text-xs text-muted-foreground"
              style={{ borderRadius: `${borderRadius}px` }}
            >
              Button
            </div>
            <div
              className="flex-1 h-12 bg-card border border-border flex items-center justify-center text-xs text-muted-foreground"
              style={{ borderRadius: `${borderRadius}px` }}
            >
              Card
            </div>
            <div
              className="flex-1 h-12 bg-muted flex items-center justify-center text-xs text-muted-foreground"
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
        "flex-1 flex flex-col items-center gap-2 p-4 rounded-lg border-2 transition-all",
        isActive
          ? "border-primary bg-primary/5"
          : "border-border hover:border-primary/50 hover:bg-muted/50",
      )}
    >
      <HugeiconsIcon
        icon={icon}
        className={cn("size-5", isActive ? "text-primary" : "text-muted-foreground")}
      />
      <span className={cn("text-xs font-medium", isActive && "text-primary")}>{label}</span>
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
    <div className="space-y-6">
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
    <div className="flex items-center justify-between py-3 border-b border-border/50">
      <div className="space-y-0.5">
        <Label className="text-sm font-medium">{label}</Label>
        <p className="text-xs text-muted-foreground">{description}</p>
      </div>
      <Switch checked={checked} onCheckedChange={onCheckedChange} />
    </div>
  )
}

// ============================================================================
// MAIN DIALOG
// ============================================================================

export function SettingsDialog({ open, onOpenChange }: SettingsDialogProps) {
  const [activeSection, setActiveSection] = useState<SettingsSection>("appearance")

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        showCloseButton={false}
        className="w-[1040px] max-w-[96vw] h-[640px] max-h-[85vh] p-0 flex flex-col overflow-hidden"
      >
        {/* Header */}
        <div className="shrink-0 px-6 py-4 border-b flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center size-8 rounded-lg bg-primary/10">
              <HugeiconsIcon icon={Settings01Icon} className="size-4 text-primary" />
            </div>
            <div>
              <h2 className="text-base font-semibold">Settings</h2>
              <p className="text-xs text-muted-foreground">Customize CADHY</p>
            </div>
          </div>
          <Button variant="ghost" size="sm" onClick={() => onOpenChange(false)}>
            Close
          </Button>
        </div>

        {/* Content */}
        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <aside className="w-56 border-r p-3 shrink-0">
            <div className="space-y-1">
              {SIDEBAR_ITEMS.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => setActiveSection(item.id)}
                  className={cn(
                    "w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-all",
                    activeSection === item.id
                      ? "bg-primary text-primary-foreground"
                      : "text-muted-foreground hover:bg-muted hover:text-foreground",
                  )}
                >
                  <HugeiconsIcon icon={item.icon} className="size-4" />
                  {item.label}
                </button>
              ))}
            </div>
          </aside>

          {/* Main Content */}
          <main className="flex-1 overflow-hidden min-w-0">
            <ScrollArea className="h-full">
              <div className="p-8 w-full">
                <h3 className="text-lg font-semibold mb-1">
                  {SIDEBAR_ITEMS.find((item) => item.id === activeSection)?.label}
                </h3>
                <p className="text-sm text-muted-foreground mb-6">
                  {activeSection === "appearance" && "Customize the look and feel"}
                  {activeSection === "viewport" && "Configure 3D viewport options"}
                </p>

                {activeSection === "appearance" && <AppearanceTab />}
                {activeSection === "viewport" && <ViewportTab />}
              </div>
            </ScrollArea>
          </main>
        </div>
      </DialogContent>
    </Dialog>
  )
}
