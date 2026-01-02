/**
 * ScenePanel - Displays scene objects hierarchy (outliner)
 *
 * Features:
 * - List all objects in the scene
 * - Select/deselect objects
 * - Add primitives
 * - Delete objects
 */

import { useEffect, useCallback } from "react"
import { useSceneStore } from "@/stores/scene-store"
import { Button, cn, ScrollArea, ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger, ContextMenuSeparator } from "@cadhy/ui"
import {
  Add01Icon,
  CubeIcon,
  CircleIcon,
  Cylinder01Icon,
  Delete02Icon,
  ViewIcon,
  ViewOffIcon,
} from "@hugeicons/core-free-icons"
import { HugeiconsIcon } from "@hugeicons/react"

export function ScenePanel() {
  const {
    objects,
    selection,
    isLoading,
    fetchObjects,
    fetchSelection,
    addCube,
    addSphere,
    addCylinder,
    removeObject,
    select,
    deselectAll,
    toggleSelection,
  } = useSceneStore()

  // Fetch objects on mount
  useEffect(() => {
    fetchObjects()
    fetchSelection()
  }, [fetchObjects, fetchSelection])

  const handleAddCube = useCallback(async () => {
    await addCube()
  }, [addCube])

  const handleAddSphere = useCallback(async () => {
    await addSphere()
  }, [addSphere])

  const handleAddCylinder = useCallback(async () => {
    await addCylinder()
  }, [addCylinder])

  const handleDelete = useCallback(async (id: string) => {
    await removeObject(id)
  }, [removeObject])

  const handleSelect = useCallback(async (id: string, extend: boolean) => {
    if (extend) {
      await toggleSelection(id)
    } else {
      await select([id], false)
    }
  }, [select, toggleSelection])

  const handleClickBackground = useCallback(async () => {
    await deselectAll()
  }, [deselectAll])

  if (objects.length === 0) {
    return (
      <div className="flex flex-col h-full">
        {/* Toolbar */}
        <div className="flex items-center gap-1 p-1.5 border-b border-border">
          <Button variant="ghost" size="icon" className="size-6" onClick={handleAddCube} title="Add Cube">
            <HugeiconsIcon icon={CubeIcon} size={14} />
          </Button>
          <Button variant="ghost" size="icon" className="size-6" onClick={handleAddSphere} title="Add Sphere">
            <HugeiconsIcon icon={CircleIcon} size={14} />
          </Button>
          <Button variant="ghost" size="icon" className="size-6" onClick={handleAddCylinder} title="Add Cylinder">
            <HugeiconsIcon icon={Cylinder01Icon} size={14} />
          </Button>
        </div>

        {/* Empty state */}
        <div className="flex-1 flex items-center justify-center p-4">
          <div className="text-center">
            <p className="text-sm text-muted-foreground mb-2">No objects in scene</p>
            <Button variant="outline" size="sm" onClick={handleAddCube}>
              <HugeiconsIcon icon={Add01Icon} size={14} className="mr-1" />
              Add Cube
            </Button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center gap-1 p-1.5 border-b border-border">
        <Button variant="ghost" size="icon" className="size-6" onClick={handleAddCube} title="Add Cube">
          <HugeiconsIcon icon={CubeIcon} size={14} />
        </Button>
        <Button variant="ghost" size="icon" className="size-6" onClick={handleAddSphere} title="Add Sphere">
          <HugeiconsIcon icon={CircleIcon} size={14} />
        </Button>
        <Button variant="ghost" size="icon" className="size-6" onClick={handleAddCylinder} title="Add Cylinder">
          <HugeiconsIcon icon={Cylinder01Icon} size={14} />
        </Button>
        <div className="flex-1" />
        <span className="text-[10px] text-muted-foreground">{objects.length} objects</span>
      </div>

      {/* Object list */}
      <ScrollArea className="flex-1">
        <div className="p-1" onClick={handleClickBackground}>
          {objects.map((obj) => (
            <ContextMenu key={obj.id}>
              <ContextMenuTrigger>
                <div
                  className={cn(
                    "flex items-center gap-2 px-2 py-1 rounded text-sm cursor-pointer",
                    "hover:bg-accent/50 transition-colors",
                    obj.selected && "bg-accent text-accent-foreground"
                  )}
                  onClick={(e) => {
                    e.stopPropagation()
                    handleSelect(obj.id, e.shiftKey || e.metaKey || e.ctrlKey)
                  }}
                >
                  <ObjectIcon type={obj.object_type} />
                  <span className="flex-1 truncate">{obj.name}</span>
                  {!obj.visible && (
                    <HugeiconsIcon icon={ViewOffIcon} size={12} className="text-muted-foreground" />
                  )}
                </div>
              </ContextMenuTrigger>
              <ContextMenuContent>
                <ContextMenuItem onClick={() => handleSelect(obj.id, false)}>
                  Select
                </ContextMenuItem>
                <ContextMenuSeparator />
                <ContextMenuItem onClick={() => handleDelete(obj.id)} className="text-destructive">
                  <HugeiconsIcon icon={Delete02Icon} size={14} className="mr-2" />
                  Delete
                </ContextMenuItem>
              </ContextMenuContent>
            </ContextMenu>
          ))}
        </div>
      </ScrollArea>

      {/* Selection info */}
      {selection.count > 0 && (
        <div className="p-2 border-t border-border text-[10px] text-muted-foreground">
          {selection.count} selected
        </div>
      )}
    </div>
  )
}

function ObjectIcon({ type }: { type: string }) {
  switch (type) {
    case "Mesh":
      return <HugeiconsIcon icon={CubeIcon} size={14} className="text-blue-400" />
    case "Curve":
      return <HugeiconsIcon icon={CircleIcon} size={14} className="text-green-400" />
    case "Light":
      return <HugeiconsIcon icon={CircleIcon} size={14} className="text-yellow-400" />
    case "Camera":
      return <HugeiconsIcon icon={ViewIcon} size={14} className="text-purple-400" />
    default:
      return <HugeiconsIcon icon={CubeIcon} size={14} className="text-muted-foreground" />
  }
}
