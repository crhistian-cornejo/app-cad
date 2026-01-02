"use client"

import { cn } from "@cadhy/ui/lib/utils"
import * as React from "react"
import { Panel, PanelGroup, PanelResizeHandle, type ImperativePanelHandle } from "react-resizable-panels"
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragStartEvent,
  DragOverlay,
} from "@dnd-kit/core"
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable"
import { CSS } from "@dnd-kit/utilities"

// ============================================================================
// Types
// ============================================================================

export interface DockablePanel {
  id: string
  label: string
  badge?: string
  icon?: React.ReactNode
  content: React.ReactNode
  defaultCollapsed?: boolean
}

export interface PanelTab {
  id: string
  label: string
  badge?: string
  icon?: React.ReactNode
  content: React.ReactNode
}

export interface DockableSidebarProps {
  panels: DockablePanel[]
  side: "left" | "right"
  defaultCollapsed?: boolean
  defaultSize?: number
  minSize?: number
  maxSize?: number
  onCollapsedChange?: (collapsed: boolean) => void
  onPanelFloat?: (panelId: string) => void
  onPanelsReorder?: (panels: DockablePanel[]) => void
  className?: string
}

// Icons
const ChevronDown = () => (
  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="m6 9 6 6 6-6"/>
  </svg>
)

const ChevronRight = () => (
  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="m9 18 6-6-6-6"/>
  </svg>
)

const GripVerticalIcon = () => (
  <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ transform: "rotate(90deg)" }}>
    <circle cx="9" cy="12" r="1" fill="currentColor"/>
    <circle cx="9" cy="5" r="1" fill="currentColor"/>
    <circle cx="9" cy="19" r="1" fill="currentColor"/>
    <circle cx="15" cy="12" r="1" fill="currentColor"/>
    <circle cx="15" cy="5" r="1" fill="currentColor"/>
    <circle cx="15" cy="19" r="1" fill="currentColor"/>
  </svg>
)

const DockSideIcon = () => (
  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <rect width="18" height="18" x="3" y="3" rx="2"/>
    <path d="M15 3v18"/>
    <path d="m8 9 3 3-3 3"/>
  </svg>
)

const MaximizeIcon = () => (
  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M8 3H5a2 2 0 0 0-2 2v3"/>
    <path d="M21 8V5a2 2 0 0 0-2-2h-3"/>
    <path d="M3 16v3a2 2 0 0 0 2 2h3"/>
    <path d="M16 21h3a2 2 0 0 0 2-2v-3"/>
  </svg>
)

const FloatWindowIcon = () => (
  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M15 3h6v6"/>
    <path d="m21 3-7 7"/>
    <path d="m3 21 7-7"/>
    <path d="M9 21H3v-6"/>
  </svg>
)

// ============================================================================
// DockablePanelHeader - Header for each dockable panel
// ============================================================================

interface DockablePanelHeaderProps {
  label: string
  badge?: string
  isCollapsed: boolean
  onToggleCollapse: () => void
  onFloat?: () => void
  className?: string
}

function DockablePanelHeader({
  label,
  badge,
  isCollapsed,
  onToggleCollapse,
  onFloat,
  className,
}: DockablePanelHeaderProps) {
  return (
    <div
      draggable="true"
      className={cn(
        "flex items-center h-7 px-1.5 bg-card border-b border-border shrink-0 gap-1 cursor-grab active:cursor-grabbing select-none",
        className
      )}
    >
      {/* Grip handle */}
      <div className="text-muted-foreground/50 p-0.5">
        <GripVerticalIcon />
      </div>

      {/* Collapse toggle + label */}
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation()
          onToggleCollapse()
        }}
        className="flex items-center gap-1 text-muted-foreground hover:text-foreground transition-colors"
      >
        {isCollapsed ? <ChevronRight /> : <ChevronDown />}
        <span className="text-[11px] font-medium uppercase tracking-wider">
          {label}
        </span>
      </button>

      {/* Badge */}
      {badge && (
        <span className="px-1 py-0.5 text-[8px] font-medium uppercase bg-muted/80 text-muted-foreground rounded">
          {badge}
        </span>
      )}

      {/* Actions */}
      <div className="ml-auto flex items-center gap-0.5">
        {onFloat && (
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation()
              onFloat()
            }}
            className="p-0.5 text-muted-foreground/50 hover:text-foreground transition-colors"
            title="Float panel"
          >
            <DockSideIcon />
          </button>
        )}
        <button
          type="button"
          className="p-0.5 text-muted-foreground/50 hover:text-foreground transition-colors"
          title="Maximize"
        >
          <MaximizeIcon />
        </button>
      </div>
    </div>
  )
}

// ============================================================================
// SortablePanelItem - Draggable expanded panel
// ============================================================================

interface SortablePanelItemProps {
  panel: DockablePanel
  onCollapse: () => void
  onFloat?: () => void
}

function SortablePanelItem({
  panel,
  onCollapse,
  onFloat,
}: SortablePanelItemProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: panel.id })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={cn(
        "flex flex-col bg-card flex-1 min-h-0",
        isDragging && "z-50 shadow-lg"
      )}
    >
      {/* Header - click to collapse entire panel */}
      <div className="flex items-center h-7 px-1.5 bg-card border-b border-border shrink-0 gap-1">
        {/* Drag handle */}
        <span
          {...attributes}
          {...listeners}
          title="Drag to reorder"
          className="text-muted-foreground/50 hover:text-muted-foreground cursor-grab active:cursor-grabbing p-0.5"
        >
          <GripVerticalIcon />
        </span>

        {/* Collapse button - collapses entire panel to tab */}
        <button
          type="button"
          onClick={onCollapse}
          className="flex items-center gap-1 text-muted-foreground hover:text-foreground transition-colors"
        >
          <ChevronDown />
          <span className="text-[11px] font-medium uppercase tracking-wider">
            {panel.label}
          </span>
        </button>

        {/* Badge */}
        {panel.badge && (
          <span className="px-1 py-0.5 text-[8px] font-medium uppercase bg-muted/80 text-muted-foreground rounded">
            {panel.badge}
          </span>
        )}

        {/* Actions */}
        <div className="ml-auto flex items-center gap-0.5">
          {onFloat && (
            <button
              type="button"
              onClick={onFloat}
              className="p-0.5 text-muted-foreground/50 hover:text-foreground transition-colors"
              title="Float as window"
            >
              <FloatWindowIcon />
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto min-h-0">
        {panel.content}
      </div>
    </div>
  )
}

// ============================================================================
// DockableSidebar - Sidebar with expandable/collapsible panels
// ============================================================================

export function DockableSidebar({
  panels: initialPanels,
  side,
  defaultCollapsed = false,
  defaultSize = 20,
  minSize = 15,
  maxSize = 40,
  onCollapsedChange,
  onPanelFloat,
  onPanelsReorder,
  className,
}: DockableSidebarProps) {
  const panelRef = React.useRef<ImperativePanelHandle>(null)
  const [panels, setPanels] = React.useState(initialPanels)
  // Collapsed panels = shown as vertical tabs, Expanded = shown with content
  const [collapsedPanelIds, setCollapsedPanelIds] = React.useState<Set<string>>(
    new Set(initialPanels.filter((p) => p.defaultCollapsed).map((p) => p.id))
  )
  const [activeId, setActiveId] = React.useState<string | null>(null)

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  )

  // Separate expanded and collapsed panels
  const expandedPanels = panels.filter((p) => !collapsedPanelIds.has(p.id))
  const collapsedPanels = panels.filter((p) => collapsedPanelIds.has(p.id))

  // All panels collapsed = sidebar shows only vertical tabs
  const allCollapsed = expandedPanels.length === 0

  const collapsePanel = React.useCallback((panelId: string) => {
    setCollapsedPanelIds((prev) => new Set([...prev, panelId]))
  }, [])

  const expandPanel = React.useCallback((panelId: string) => {
    setCollapsedPanelIds((prev) => {
      const next = new Set(prev)
      next.delete(panelId)
      return next
    })
  }, [])

  const handleDragStart = React.useCallback((event: DragStartEvent) => {
    setActiveId(event.active.id as string)
  }, [])

  const handleDragEnd = React.useCallback((event: DragEndEvent) => {
    const { active, over } = event
    setActiveId(null)

    if (over && active.id !== over.id) {
      setPanels((items) => {
        const oldIndex = items.findIndex((item) => item.id === active.id)
        const newIndex = items.findIndex((item) => item.id === over.id)
        const newPanels = arrayMove(items, oldIndex, newIndex)
        onPanelsReorder?.(newPanels)
        return newPanels
      })
    }
  }, [onPanelsReorder])

  // When all collapsed, use minimal width (~24px = ~1.5% of viewport)
  const collapsedSize = 1.5

  return (
    <Panel
      ref={panelRef}
      id={`${side}-panel`}
      order={side === "left" ? 1 : 3}
      defaultSize={allCollapsed ? collapsedSize : defaultSize}
      minSize={allCollapsed ? collapsedSize : minSize}
      maxSize={allCollapsed ? collapsedSize : maxSize}
      collapsible={false}
      className={cn("relative", className)}
    >
      <div className={cn("flex h-full", side === "right" && "flex-row-reverse")}>
        {/* Vertical tabs for COLLAPSED panels */}
        {collapsedPanels.length > 0 && (
          <div
            className={cn(
              "flex flex-col bg-card shrink-0",
              side === "left" ? "border-r border-border" : "border-l border-border",
              allCollapsed ? "w-full" : "w-6"
            )}
          >
            {collapsedPanels.map((panel) => (
              <button
                key={panel.id}
                type="button"
                onClick={() => expandPanel(panel.id)}
                className="flex flex-col items-center py-2 px-1 transition-colors hover:bg-accent/50 gap-2"
                title={`Expand ${panel.label}`}
              >
                {/* Grip icon */}
                <span className="text-muted-foreground/40">
                  <GripVerticalIcon />
                </span>
                {/* Vertical text */}
                <span
                  className="text-[9px] font-medium uppercase tracking-[0.15em] whitespace-nowrap text-muted-foreground"
                  style={{
                    writingMode: "vertical-rl",
                    textOrientation: "mixed",
                  }}
                >
                  {panel.label}
                </span>
              </button>
            ))}
          </div>
        )}

        {/* Expanded panels with content */}
        {expandedPanels.length > 0 && (
          <div className="flex-1 flex flex-col h-full bg-card overflow-hidden">
            <DndContext
              sensors={sensors}
              collisionDetection={closestCenter}
              onDragStart={handleDragStart}
              onDragEnd={handleDragEnd}
            >
              <SortableContext
                items={expandedPanels.map((p) => p.id)}
                strategy={verticalListSortingStrategy}
              >
                {expandedPanels.map((panel, index) => (
                  <React.Fragment key={panel.id}>
                    {index > 0 && (
                      <div className="h-px bg-border shrink-0" />
                    )}
                    <SortablePanelItem
                      panel={panel}
                      onCollapse={() => collapsePanel(panel.id)}
                      onFloat={onPanelFloat ? () => onPanelFloat(panel.id) : undefined}
                    />
                  </React.Fragment>
                ))}
              </SortableContext>
              <DragOverlay>
                {activeId ? (
                  <div className="bg-card border border-border rounded shadow-lg p-2 opacity-90">
                    <span className="text-xs font-medium uppercase">
                      {panels.find((p) => p.id === activeId)?.label}
                    </span>
                  </div>
                ) : null}
              </DragOverlay>
            </DndContext>
          </div>
        )}
      </div>
    </Panel>
  )
}

// ============================================================================
// FloatingPanel - Draggable floating panel
// ============================================================================

export interface FloatingPanelProps {
  id: string
  label: string
  badge?: string
  content: React.ReactNode
  defaultPosition?: { x: number; y: number }
  defaultSize?: { width: number; height: number }
  onClose?: () => void
  onDock?: () => void
}

export function FloatingPanel({
  id,
  label,
  badge,
  content,
  defaultPosition = { x: 100, y: 100 },
  defaultSize = { width: 280, height: 400 },
  onClose,
  onDock,
}: FloatingPanelProps) {
  const [position, setPosition] = React.useState(defaultPosition)
  const [isDragging, setIsDragging] = React.useState(false)
  const dragOffset = React.useRef({ x: 0, y: 0 })

  const handleMouseDown = React.useCallback((e: React.MouseEvent) => {
    setIsDragging(true)
    dragOffset.current = {
      x: e.clientX - position.x,
      y: e.clientY - position.y,
    }
  }, [position])

  React.useEffect(() => {
    if (!isDragging) return

    const handleMouseMove = (e: MouseEvent) => {
      setPosition({
        x: e.clientX - dragOffset.current.x,
        y: e.clientY - dragOffset.current.y,
      })
    }

    const handleMouseUp = () => {
      setIsDragging(false)
    }

    document.addEventListener("mousemove", handleMouseMove)
    document.addEventListener("mouseup", handleMouseUp)

    return () => {
      document.removeEventListener("mousemove", handleMouseMove)
      document.removeEventListener("mouseup", handleMouseUp)
    }
  }, [isDragging])

  return (
    <div
      className="fixed z-50 bg-card border border-border rounded-lg shadow-2xl overflow-hidden"
      style={{
        left: position.x,
        top: position.y,
        width: defaultSize.width,
        height: defaultSize.height,
      }}
    >
      {/* Header */}
      <div
        className="flex items-center h-7 px-2 bg-muted/50 border-b border-border cursor-grab active:cursor-grabbing"
        onMouseDown={handleMouseDown}
      >
        <span className="text-[11px] font-medium uppercase tracking-wider text-foreground">
          {label}
        </span>
        {badge && (
          <span className="ml-2 px-1 py-0.5 text-[8px] font-medium uppercase bg-muted text-muted-foreground rounded">
            {badge}
          </span>
        )}
        <div className="ml-auto flex items-center gap-1">
          {onDock && (
            <button
              type="button"
              onClick={onDock}
              className="p-0.5 text-muted-foreground hover:text-foreground transition-colors"
              title="Dock panel"
            >
              <DockSideIcon />
            </button>
          )}
          {onClose && (
            <button
              type="button"
              onClick={onClose}
              className="p-0.5 text-muted-foreground hover:text-foreground transition-colors"
              title="Close"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M18 6 6 18M6 6l12 12"/>
              </svg>
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="h-[calc(100%-28px)] overflow-auto">
        {content}
      </div>
    </div>
  )
}

// ============================================================================
// DockableLayout - Full layout with sidebars and floating panels
// ============================================================================

export interface DockableLayoutProps {
  leftPanels?: DockablePanel[]
  rightPanels?: DockablePanel[]
  floatingPanels?: FloatingPanelProps[]
  leftDefaultCollapsed?: boolean
  rightDefaultCollapsed?: boolean
  leftDefaultSize?: number
  rightDefaultSize?: number
  children: React.ReactNode
  className?: string
}

export function DockableLayout({
  leftPanels,
  rightPanels,
  floatingPanels,
  leftDefaultCollapsed = false,
  rightDefaultCollapsed = false,
  leftDefaultSize = 20,
  rightDefaultSize = 20,
  children,
  className,
}: DockableLayoutProps) {
  return (
    <>
      <PanelGroup direction="horizontal" className={cn("h-full", className)}>
        {leftPanels && leftPanels.length > 0 && (
          <>
            <DockableSidebar
              panels={leftPanels}
              side="left"
              defaultCollapsed={leftDefaultCollapsed}
              defaultSize={leftDefaultSize}
            />
            <PanelResizeHandle className="w-px bg-border hover:bg-accent data-[resize-handle-active]:bg-accent transition-colors cursor-col-resize" />
          </>
        )}

        <Panel id="center" order={2} defaultSize={60} minSize={30}>
          {children}
        </Panel>

        {rightPanels && rightPanels.length > 0 && (
          <>
            <PanelResizeHandle className="w-px bg-border hover:bg-accent data-[resize-handle-active]:bg-accent transition-colors cursor-col-resize" />
            <DockableSidebar
              panels={rightPanels}
              side="right"
              defaultCollapsed={rightDefaultCollapsed}
              defaultSize={rightDefaultSize}
            />
          </>
        )}
      </PanelGroup>

      {/* Floating panels */}
      {floatingPanels?.map((panel) => (
        <FloatingPanel key={panel.id} {...panel} />
      ))}
    </>
  )
}

// ============================================================================
// Legacy exports for backwards compatibility
// ============================================================================

export interface CollapsibleSidePanelProps {
  tabs: PanelTab[]
  side: "left" | "right"
  defaultCollapsed?: boolean
  defaultActiveTab?: string
  defaultSize?: number
  minSize?: number
  maxSize?: number
  onCollapsedChange?: (collapsed: boolean) => void
  onActiveTabChange?: (tabId: string) => void
  className?: string
}

export function CollapsibleSidePanel(props: CollapsibleSidePanelProps) {
  // Convert tabs to panels format
  const panels: DockablePanel[] = props.tabs.map((tab) => ({
    id: tab.id,
    label: tab.label,
    badge: tab.badge,
    icon: tab.icon,
    content: tab.content,
  }))

  return (
    <DockableSidebar
      panels={panels}
      side={props.side}
      defaultCollapsed={props.defaultCollapsed}
      defaultSize={props.defaultSize}
      minSize={props.minSize}
      maxSize={props.maxSize}
      onCollapsedChange={props.onCollapsedChange}
    />
  )
}

export interface CollapsiblePanelLayoutProps {
  leftPanel?: {
    tabs: PanelTab[]
    defaultCollapsed?: boolean
    defaultActiveTab?: string
    defaultSize?: number
  }
  rightPanel?: {
    tabs: PanelTab[]
    defaultCollapsed?: boolean
    defaultActiveTab?: string
    defaultSize?: number
  }
  children: React.ReactNode
  className?: string
}

export function CollapsiblePanelLayout({
  leftPanel,
  rightPanel,
  children,
  className,
}: CollapsiblePanelLayoutProps) {
  const leftPanels = leftPanel?.tabs.map((tab) => ({
    id: tab.id,
    label: tab.label,
    badge: tab.badge,
    icon: tab.icon,
    content: tab.content,
  }))

  const rightPanels = rightPanel?.tabs.map((tab) => ({
    id: tab.id,
    label: tab.label,
    badge: tab.badge,
    icon: tab.icon,
    content: tab.content,
  }))

  return (
    <DockableLayout
      leftPanels={leftPanels}
      rightPanels={rightPanels}
      leftDefaultCollapsed={leftPanel?.defaultCollapsed}
      rightDefaultCollapsed={rightPanel?.defaultCollapsed}
      leftDefaultSize={leftPanel?.defaultSize}
      rightDefaultSize={rightPanel?.defaultSize}
      className={className}
    >
      {children}
    </DockableLayout>
  )
}
