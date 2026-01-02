/**
 * Command Palette - CADHY
 *
 * Simplified command palette for basic viewport operations.
 */

import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandGroupLabel,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
  CommandShortcut,
  Kbd,
  KbdGroup,
} from "@cadhy/ui"
import {
  ArrowTurnBackwardIcon,
  ArrowTurnForwardIcon,
  CubeIcon,
  File01Icon,
  FolderOpenIcon,
  GridIcon,
  Home01Icon,
  MaximizeScreenIcon,
  Moon02Icon,
  Search01Icon,
  Sun03Icon,
} from "@hugeicons/core-free-icons"
import { HugeiconsIcon } from "@hugeicons/react"
import * as React from "react"
import { useCallback, useEffect, useMemo, useState } from "react"
import { useLayoutStore } from "@/stores/layout-store"
import { useViewportStore } from "@/stores/viewport-store"
import { viewportApi, sceneApi } from "@/lib/tauri"

// ============================================================================
// TYPES
// ============================================================================

type CommandCategory = "file" | "edit" | "view" | "camera" | "theme" | "tools"

interface CommandItemDef {
  id: string
  label: string
  description?: string
  icon?: typeof File01Icon
  shortcut?: string
  action: () => void | Promise<void>
  category: CommandCategory
  keywords?: string[]
  disabled?: boolean
}

interface GroupedCommands {
  value: CommandCategory
  label: string
  items: CommandItemDef[]
}

interface CommandPaletteProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onOpenSettings?: () => void
  onOpenShortcuts?: () => void
  onNewProject?: () => void
  onOpenProject?: () => void
}

// ============================================================================
// CATEGORY CONFIG
// ============================================================================

const CATEGORY_ORDER: CommandCategory[] = ["file", "edit", "view", "camera", "theme", "tools"]

const CATEGORY_LABELS: Record<CommandCategory, string> = {
  file: "File",
  edit: "Edit",
  view: "View",
  camera: "Camera",
  theme: "Theme",
  tools: "Tools",
}

// ============================================================================
// COMPONENT
// ============================================================================

export function CommandPalette({
  open,
  onOpenChange,
  onOpenSettings,
  onOpenShortcuts,
  onNewProject,
  onOpenProject,
}: CommandPaletteProps) {
  const [search, setSearch] = useState("")
  const [recentCommandIds, setRecentCommandIds] = useState<string[]>([])

  const modKey = "⌘"
  const shiftKey = "⇧"

  // ========== STORE HOOKS ==========

  // Viewport store
  const { viewMode, setViewMode, frameAll, resetCamera } = useViewportStore()

  // Layout store
  const { togglePanel, theme, setTheme } = useLayoutStore()

  // ========== HELPERS ==========

  const closeAndRun = useCallback(
    (action: () => void | Promise<void>, commandId?: string) => {
      return () => {
        onOpenChange(false)
        setSearch("")
        if (commandId) {
          setRecentCommandIds((prev) => {
            const filtered = prev.filter((id) => id !== commandId)
            return [commandId, ...filtered].slice(0, 5)
          })
        }
        action()
      }
    },
    [onOpenChange],
  )

  // ========== COMMANDS ==========

  const commands: CommandItemDef[] = useMemo(
    () => [
      // ===== FILE =====
      {
        id: "file.new",
        label: "New Project",
        description: "Create a new project",
        icon: File01Icon,
        shortcut: `${modKey}N`,
        category: "file",
        keywords: ["new", "create", "project"],
        action: closeAndRun(() => onNewProject?.(), "file.new"),
      },
      {
        id: "file.open",
        label: "Open Project",
        description: "Open an existing project",
        icon: FolderOpenIcon,
        shortcut: `${modKey}O`,
        category: "file",
        keywords: ["open", "load", "project"],
        action: closeAndRun(() => onOpenProject?.(), "file.open"),
      },

      // ===== EDIT =====
      {
        id: "edit.undo",
        label: "Undo",
        description: "Undo the last action",
        icon: ArrowTurnBackwardIcon,
        shortcut: `${modKey}Z`,
        category: "edit",
        keywords: ["undo", "back", "revert"],
        action: closeAndRun(() => console.log("Undo - TODO"), "edit.undo"),
      },
      {
        id: "edit.redo",
        label: "Redo",
        description: "Redo the last undone action",
        icon: ArrowTurnForwardIcon,
        shortcut: `${modKey}${shiftKey}Z`,
        category: "edit",
        keywords: ["redo", "forward"],
        action: closeAndRun(() => console.log("Redo - TODO"), "edit.redo"),
      },

      // ===== VIEW =====
      {
        id: "view.solid",
        label: "Solid View",
        description: "Switch to solid rendering mode",
        icon: CubeIcon,
        shortcut: "Z",
        category: "view",
        keywords: ["solid", "render", "mode"],
        action: closeAndRun(() => setViewMode("solid"), "view.solid"),
      },
      {
        id: "view.wireframe",
        label: "Wireframe View",
        description: "Switch to wireframe rendering mode",
        icon: GridIcon,
        shortcut: "Z",
        category: "view",
        keywords: ["wireframe", "render", "mode"],
        action: closeAndRun(() => setViewMode("wireframe"), "view.wireframe"),
      },

      // ===== CAMERA =====
      {
        id: "camera.frameAll",
        label: "Frame All",
        description: "Zoom to fit all objects",
        icon: MaximizeScreenIcon,
        shortcut: "Home",
        category: "camera",
        keywords: ["fit", "zoom", "all", "scene"],
        action: closeAndRun(() => frameAll(), "camera.frameAll"),
      },
      {
        id: "camera.reset",
        label: "Reset Camera",
        description: "Reset camera to default position",
        icon: Home01Icon,
        category: "camera",
        keywords: ["reset", "camera", "default"],
        action: closeAndRun(() => resetCamera(), "camera.reset"),
      },

      // ===== THEME =====
      {
        id: "theme.light",
        label: "Light Theme",
        description: theme === "light" ? "Currently active" : "Switch to light mode",
        icon: Sun03Icon,
        category: "theme",
        keywords: ["light", "theme", "bright", "day"],
        disabled: theme === "light",
        action: closeAndRun(() => setTheme("light"), "theme.light"),
      },
      {
        id: "theme.dark",
        label: "Dark Theme",
        description: theme === "dark" ? "Currently active" : "Switch to dark mode",
        icon: Moon02Icon,
        category: "theme",
        keywords: ["dark", "theme", "night", "mode"],
        disabled: theme === "dark",
        action: closeAndRun(() => setTheme("dark"), "theme.dark"),
      },

      // ===== TOOLS =====
      {
        id: "tools.addCube",
        label: "Add Cube",
        description: "Add a cube to the scene",
        icon: CubeIcon,
        category: "tools",
        keywords: ["cube", "add", "primitive", "shape"],
        action: closeAndRun(async () => {
          try {
            await sceneApi.addCube("Cube", 1.0)
          } catch (e) {
            console.error("Failed to add cube:", e)
          }
        }, "tools.addCube"),
      },
    ],
    [
      theme,
      setTheme,
      setViewMode,
      frameAll,
      resetCamera,
      onNewProject,
      onOpenProject,
      onOpenChange,
      closeAndRun,
    ],
  )

  // ========== FILTERING ==========

  const filteredCommands = useMemo(() => {
    if (!search) return commands

    const searchLower = search.toLowerCase()
    return commands.filter(
      (cmd) =>
        cmd.label.toLowerCase().includes(searchLower) ||
        cmd.description?.toLowerCase().includes(searchLower) ||
        cmd.keywords?.some((kw) => kw.toLowerCase().includes(searchLower)),
    )
  }, [commands, search])

  // ========== GROUPING ==========

  const groupedCommands = useMemo(() => {
    const groups: GroupedCommands[] = []

    for (const category of CATEGORY_ORDER) {
      const items = filteredCommands.filter((cmd) => cmd.category === category)
      if (items.length > 0) {
        groups.push({
          value: category,
          label: CATEGORY_LABELS[category],
          items,
        })
      }
    }

    return groups
  }, [filteredCommands])

  // ========== RENDER ==========

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <Command>
        <CommandInput
          placeholder="Type a command or search..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <CommandList>
          <CommandEmpty>No commands found.</CommandEmpty>

          {groupedCommands.map((group) => (
            <CommandGroup key={group.value}>
              <CommandGroupLabel>{group.label}</CommandGroupLabel>
              {group.items.map((cmd) => (
                <CommandItem
                  key={cmd.id}
                  value={cmd.label}
                  onSelect={cmd.action}
                  disabled={cmd.disabled}
                >
                  <div className="flex items-center gap-2">
                    {cmd.icon && <HugeiconsIcon icon={cmd.icon} size={16} />}
                    <div className="flex-1">
                      <div className="font-medium">{cmd.label}</div>
                      {cmd.description && (
                        <div className="text-sm text-muted-foreground">{cmd.description}</div>
                      )}
                    </div>
                    {cmd.shortcut && (
                      <CommandShortcut>
                        <KbdGroup>
                          {cmd.shortcut.split("").map((key, i) => (
                            <Kbd key={i}>{key}</Kbd>
                          ))}
                        </KbdGroup>
                      </CommandShortcut>
                    )}
                  </div>
                </CommandItem>
              ))}
              {group.items.length > 0 && <CommandSeparator />}
            </CommandGroup>
          ))}
        </CommandList>
      </Command>
    </CommandDialog>
  )
}
