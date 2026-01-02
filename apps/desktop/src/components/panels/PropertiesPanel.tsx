/**
 * PropertiesPanel - Displays and edits properties of selected objects
 *
 * Features:
 * - Transform controls (position, rotation, scale)
 * - Object info
 */

import { useEffect, useState, useCallback } from "react"
import { useSceneStore } from "@/stores/scene-store"
import { Input, Label, Button, Separator } from "@cadhy/ui"
import { ThreeDMoveIcon, RotateClockwiseIcon, Resize01Icon, RefreshIcon } from "@hugeicons/core-free-icons"
import { HugeiconsIcon } from "@hugeicons/react"
import type { TransformDto } from "@/lib/tauri"

export function PropertiesPanel() {
  const { objects, selection, getTransform, setTransform } = useSceneStore()

  const [transform, setLocalTransform] = useState<TransformDto | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  // Get the first selected object
  const selectedId = selection.ids[0]
  const selectedObject = objects.find((obj) => obj.id === selectedId)

  // Fetch transform when selection changes
  useEffect(() => {
    if (selectedId) {
      setIsLoading(true)
      getTransform(selectedId).then((t) => {
        setLocalTransform(t)
        setIsLoading(false)
      })
    } else {
      setLocalTransform(null)
    }
  }, [selectedId, getTransform])

  const handlePositionChange = useCallback(
    async (axis: 0 | 1 | 2, value: number) => {
      if (!transform || !selectedId) return
      const newTransform: TransformDto = {
        ...transform,
        position: [...transform.position] as [number, number, number],
      }
      newTransform.position[axis] = value
      setLocalTransform(newTransform)
      await setTransform(selectedId, newTransform)
    },
    [transform, selectedId, setTransform]
  )

  const handleScaleChange = useCallback(
    async (axis: 0 | 1 | 2, value: number) => {
      if (!transform || !selectedId) return
      const newTransform: TransformDto = {
        ...transform,
        scale: [...transform.scale] as [number, number, number],
      }
      newTransform.scale[axis] = value
      setLocalTransform(newTransform)
      await setTransform(selectedId, newTransform)
    },
    [transform, selectedId, setTransform]
  )

  const handleResetTransform = useCallback(async () => {
    if (!selectedId) return
    const defaultTransform: TransformDto = {
      position: [0, 0, 0],
      rotation: [0, 0, 0, 1],
      scale: [1, 1, 1],
    }
    setLocalTransform(defaultTransform)
    await setTransform(selectedId, defaultTransform)
  }, [selectedId, setTransform])

  if (!selectedObject) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground text-center">
          Select an object to view properties
        </p>
      </div>
    )
  }

  if (selection.count > 1) {
    return (
      <div className="p-3">
        <p className="text-sm text-muted-foreground">
          {selection.count} objects selected
        </p>
        <p className="text-xs text-muted-foreground/60 mt-1">
          Multi-object editing not yet supported
        </p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      {/* Object Info */}
      <div className="p-3 border-b border-border">
        <div className="flex items-center justify-between">
          <div>
            <h4 className="text-sm font-medium">{selectedObject.name}</h4>
            <p className="text-[10px] text-muted-foreground uppercase">
              {selectedObject.object_type}
            </p>
          </div>
          <Button
            variant="ghost"
            size="icon"
            className="size-6"
            onClick={handleResetTransform}
            title="Reset Transform"
          >
            <HugeiconsIcon icon={RefreshIcon} size={14} />
          </Button>
        </div>
      </div>

      {/* Transform Section */}
      {transform && (
        <div className="p-3 space-y-4">
          {/* Position */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <HugeiconsIcon icon={ThreeDMoveIcon} size={14} className="text-muted-foreground" />
              <Label className="text-[10px] uppercase tracking-wider">Position</Label>
            </div>
            <div className="grid grid-cols-3 gap-2">
              <TransformInput
                label="X"
                value={transform.position[0]}
                onChange={(v) => handlePositionChange(0, v)}
                color="text-red-400"
              />
              <TransformInput
                label="Y"
                value={transform.position[1]}
                onChange={(v) => handlePositionChange(1, v)}
                color="text-green-400"
              />
              <TransformInput
                label="Z"
                value={transform.position[2]}
                onChange={(v) => handlePositionChange(2, v)}
                color="text-blue-400"
              />
            </div>
          </div>

          <Separator />

          {/* Rotation (displayed as euler angles) */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <HugeiconsIcon icon={RotateClockwiseIcon} size={14} className="text-muted-foreground" />
              <Label className="text-[10px] uppercase tracking-wider">Rotation</Label>
            </div>
            <div className="grid grid-cols-3 gap-2">
              <TransformInput
                label="X"
                value={quaternionToEulerX(transform.rotation)}
                onChange={() => {}}
                color="text-red-400"
                disabled
              />
              <TransformInput
                label="Y"
                value={quaternionToEulerY(transform.rotation)}
                onChange={() => {}}
                color="text-green-400"
                disabled
              />
              <TransformInput
                label="Z"
                value={quaternionToEulerZ(transform.rotation)}
                onChange={() => {}}
                color="text-blue-400"
                disabled
              />
            </div>
            <p className="text-[9px] text-muted-foreground/60 mt-1">
              Rotation editing coming soon
            </p>
          </div>

          <Separator />

          {/* Scale */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <HugeiconsIcon icon={Resize01Icon} size={14} className="text-muted-foreground" />
              <Label className="text-[10px] uppercase tracking-wider">Scale</Label>
            </div>
            <div className="grid grid-cols-3 gap-2">
              <TransformInput
                label="X"
                value={transform.scale[0]}
                onChange={(v) => handleScaleChange(0, v)}
                color="text-red-400"
              />
              <TransformInput
                label="Y"
                value={transform.scale[1]}
                onChange={(v) => handleScaleChange(1, v)}
                color="text-green-400"
              />
              <TransformInput
                label="Z"
                value={transform.scale[2]}
                onChange={(v) => handleScaleChange(2, v)}
                color="text-blue-400"
              />
            </div>
          </div>
        </div>
      )}

      {/* Object ID */}
      <div className="mt-auto p-2 border-t border-border">
        <p className="text-[9px] text-muted-foreground/50 font-mono truncate">
          ID: {selectedId}
        </p>
      </div>
    </div>
  )
}

interface TransformInputProps {
  label: string
  value: number
  onChange: (value: number) => void
  color?: string
  disabled?: boolean
}

function TransformInput({ label, value, onChange, color, disabled }: TransformInputProps) {
  const [localValue, setLocalValue] = useState(value.toFixed(3))

  useEffect(() => {
    setLocalValue(value.toFixed(3))
  }, [value])

  const handleBlur = () => {
    const num = parseFloat(localValue)
    if (!isNaN(num)) {
      onChange(num)
    } else {
      setLocalValue(value.toFixed(3))
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleBlur()
    }
  }

  return (
    <div className="flex flex-col gap-1">
      <Label className={`text-[9px] ${color || "text-muted-foreground"}`}>{label}</Label>
      <Input
        type="text"
        value={localValue}
        onChange={(e) => setLocalValue(e.target.value)}
        onBlur={handleBlur}
        onKeyDown={handleKeyDown}
        disabled={disabled}
        className="h-7 text-xs font-mono"
      />
    </div>
  )
}

// Convert quaternion to euler angles (degrees)
function quaternionToEulerX(q: [number, number, number, number]): number {
  const [x, y, z, w] = q
  const sinr = 2 * (w * x + y * z)
  const cosr = 1 - 2 * (x * x + y * y)
  return Math.round(Math.atan2(sinr, cosr) * (180 / Math.PI))
}

function quaternionToEulerY(q: [number, number, number, number]): number {
  const [x, y, z, w] = q
  const sinp = 2 * (w * y - z * x)
  if (Math.abs(sinp) >= 1) {
    return Math.round((Math.sign(sinp) * Math.PI) / 2 * (180 / Math.PI))
  }
  return Math.round(Math.asin(sinp) * (180 / Math.PI))
}

function quaternionToEulerZ(q: [number, number, number, number]): number {
  const [x, y, z, w] = q
  const siny = 2 * (w * z + x * y)
  const cosy = 1 - 2 * (y * y + z * z)
  return Math.round(Math.atan2(siny, cosy) * (180 / Math.PI))
}
