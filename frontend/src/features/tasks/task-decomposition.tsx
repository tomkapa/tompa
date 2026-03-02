import { useState, useCallback } from 'react'
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core'
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { GripVertical, Pencil, GitBranch, Merge, Split, Plus, X, Check } from 'lucide-react'
import { cn } from '@/lib/utils'
import { TaskTypeIcon, type TaskType } from '@/components/ui/task-type-icon'
import { Button } from '@/components/ui/button'

export interface ProposedTask {
  id: string
  name: string
  taskType: TaskType
  scopeEstimate?: string
  /** IDs of tasks this one depends on */
  dependencies?: string[]
}

interface TaskDecompositionProps {
  proposedTasks: ProposedTask[]
  onConfirm: (tasks: ProposedTask[]) => void
  onReorder: (tasks: ProposedTask[]) => void
  onMerge: (ids: string[]) => void
  onSplit: (id: string, subtasks: string[]) => void
  onEditTask: (id: string, name: string) => void
}

// ─── Split Editor ────────────────────────────────────────────────────────────

interface SplitEditorProps {
  task: ProposedTask
  onApply: (subtasks: string[]) => void
  onCancel: () => void
}

function SplitEditor({ task, onApply, onCancel }: SplitEditorProps) {
  const [inputs, setInputs] = useState<string[]>(['', ''])

  const updateInput = (idx: number, val: string) =>
    setInputs((prev) => prev.map((v, i) => (i === idx ? val : v)))

  const addInput = () => setInputs((prev) => [...prev, ''])

  const handleApply = () => {
    const names = inputs.map((s) => s.trim()).filter(Boolean)
    if (names.length >= 2) onApply(names)
  }

  return (
    <div className="flex flex-col gap-3 rounded-lg border-2 border-primary bg-card p-4">
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold text-foreground">Split: {task.name}</span>
        <button onClick={onCancel} className="text-muted-foreground hover:text-foreground">
          <X size={16} />
        </button>
      </div>

      <div className="flex flex-col gap-2">
        {inputs.map((val, idx) => (
          <div key={idx} className="flex items-center gap-2">
            <span className="w-5 shrink-0 text-[13px] font-medium text-muted-foreground">
              {idx + 1}.
            </span>
            <input
              value={val}
              onChange={(e) => updateInput(idx, e.target.value)}
              placeholder={`Subtask ${idx + 1}`}
              className={cn(
                'flex-1 rounded-lg border border-border bg-accent px-3 py-2',
                'text-[13px] text-foreground placeholder:text-muted-foreground',
                'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring'
              )}
            />
          </div>
        ))}

        <button
          onClick={addInput}
          className="flex items-center gap-1 px-0 py-1 text-xs font-medium text-primary hover:text-primary/80"
        >
          <Plus size={14} />
          Add subtask
        </button>
      </div>

      <div className="flex justify-end gap-2">
        <Button variant="outline" size="default" onClick={onCancel}>
          Cancel
        </Button>
        <Button size="default" onClick={handleApply} disabled={inputs.filter(Boolean).length < 2}>
          Apply Split
        </Button>
      </div>
    </div>
  )
}

// ─── Sortable Task Item ───────────────────────────────────────────────────────

interface SortableTaskItemProps {
  task: ProposedTask
  selected: boolean
  onToggleSelect: () => void
  onEdit: (name: string) => void
  onSplitClick: () => void
  taskIndex: number
  totalTasks: number
}

function SortableTaskItem({
  task,
  selected,
  onToggleSelect,
  onEdit,
  taskIndex,
}: SortableTaskItemProps) {
  const [editing, setEditing] = useState(false)
  const [editValue, setEditValue] = useState(task.name)

  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: task.id,
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  const handleEditCommit = () => {
    const trimmed = editValue.trim()
    if (trimmed && trimmed !== task.name) onEdit(trimmed)
    setEditing(false)
  }

  const hasDeps = task.dependencies && task.dependencies.length > 0

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={cn(
        'flex h-[52px] items-center border-b border-border transition-colors',
        selected && 'bg-[var(--color-info)]',
        isDragging && 'opacity-50 shadow-lg'
      )}
    >
      {/* Checkbox */}
      <button
        onClick={onToggleSelect}
        className={cn(
          'flex h-full w-10 shrink-0 items-center justify-center text-muted-foreground',
          'hover:text-foreground focus-visible:outline-none'
        )}
      >
        <span
          className={cn(
            'flex h-4 w-4 items-center justify-center rounded border border-border',
            selected && 'border-[var(--color-info-foreground)] bg-[var(--color-info-foreground)]'
          )}
        >
          {selected && <Check size={10} className="text-[var(--color-info)]" />}
        </span>
      </button>

      {/* Drag handle */}
      <div
        {...attributes}
        {...listeners}
        className="flex h-full w-8 shrink-0 cursor-grab items-center justify-center text-muted-foreground active:cursor-grabbing"
      >
        <GripVertical size={16} />
      </div>

      {/* Task content */}
      <div className="flex flex-1 min-w-0 items-center gap-[10px] px-3 h-full">
        <TaskTypeIcon type={task.taskType} />

        {editing ? (
          <input
            autoFocus
            value={editValue}
            onChange={(e) => setEditValue(e.target.value)}
            onBlur={handleEditCommit}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleEditCommit()
              if (e.key === 'Escape') {
                setEditValue(task.name)
                setEditing(false)
              }
            }}
            className={cn(
              'flex-1 min-w-0 rounded border border-primary bg-transparent px-1 py-0.5',
              'text-sm text-foreground focus-visible:outline-none'
            )}
          />
        ) : (
          <span className="flex-1 min-w-0 truncate text-sm text-foreground">{task.name}</span>
        )}

        {task.scopeEstimate && (
          <span
            className={cn(
              'inline-flex items-center rounded-full bg-accent px-2 py-0.5',
              'text-[11px] font-medium text-muted-foreground shrink-0'
            )}
          >
            {task.scopeEstimate}
          </span>
        )}

        {hasDeps && (
          <div className="flex items-center gap-1 shrink-0 text-muted-foreground">
            <GitBranch size={12} />
            <span className="text-[11px]">
              → Task {task.dependencies!.map((_, i) => taskIndex - (i + 1)).join(', ')}
            </span>
          </div>
        )}
      </div>

      {/* Edit button */}
      <button
        onClick={() => setEditing(true)}
        className={cn(
          'flex h-full w-9 shrink-0 items-center justify-center',
          'text-muted-foreground hover:text-foreground focus-visible:outline-none'
        )}
      >
        <Pencil size={14} />
      </button>
    </div>
  )
}

// ─── Task Decomposition Review ────────────────────────────────────────────────

export function TaskDecomposition({
  proposedTasks,
  onConfirm,
  onReorder,
  onMerge,
  onSplit,
  onEditTask,
}: TaskDecompositionProps) {
  const [tasks, setTasks] = useState<ProposedTask[]>(proposedTasks)
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [splitTargetId, setSplitTargetId] = useState<string | null>(null)

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  )

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event
      if (!over || active.id === over.id) return
      setTasks((prev) => {
        const oldIdx = prev.findIndex((t) => t.id === active.id)
        const newIdx = prev.findIndex((t) => t.id === over.id)
        const reordered = arrayMove(prev, oldIdx, newIdx)
        onReorder(reordered)
        return reordered
      })
    },
    [onReorder]
  )

  const toggleSelect = (id: string) =>
    setSelected((prev) => {
      const next = new Set(prev)
      next.has(id) ? next.delete(id) : next.add(id)
      return next
    })

  const handleMerge = () => {
    if (selected.size < 2) return
    onMerge([...selected])
    setSelected(new Set())
  }

  const handleSplitClick = () => {
    if (selected.size === 1) {
      setSplitTargetId([...selected][0])
    }
  }

  const handleSplitApply = (subtasks: string[]) => {
    if (!splitTargetId) return
    onSplit(splitTargetId, subtasks)
    setSplitTargetId(null)
    setSelected(new Set())
  }

  const splitTask = splitTargetId ? tasks.find((t) => t.id === splitTargetId) : null

  return (
    <div className="flex flex-col overflow-hidden rounded-2xl border border-border bg-background">
      {/* Panel header */}
      <div className="flex flex-col gap-2 border-b border-border px-5 py-4">
        <div className="flex items-center justify-between">
          <span className="text-base font-semibold leading-snug text-foreground">
            Review Task Breakdown
          </span>
          <span className="text-[13px] text-muted-foreground">{tasks.length} tasks</span>
        </div>
        <p className="text-xs leading-snug text-muted-foreground">
          Drag to reorder, select multiple to merge, or click to edit
        </p>
      </div>

      {/* Sortable task list */}
      <div className="flex flex-1 flex-col overflow-y-auto">
        <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
          <SortableContext items={tasks.map((t) => t.id)} strategy={verticalListSortingStrategy}>
            {tasks.map((task, idx) => (
              <SortableTaskItem
                key={task.id}
                task={task}
                selected={selected.has(task.id)}
                onToggleSelect={() => toggleSelect(task.id)}
                onEdit={(name) => onEditTask(task.id, name)}
                onSplitClick={handleSplitClick}
                taskIndex={idx + 1}
                totalTasks={tasks.length}
              />
            ))}
          </SortableContext>
        </DndContext>

        {/* Inline split editor */}
        {splitTask && (
          <div className="p-4">
            <SplitEditor
              task={splitTask}
              onApply={handleSplitApply}
              onCancel={() => setSplitTargetId(null)}
            />
          </div>
        )}
      </div>

      {/* Action bar */}
      <div className="flex items-center justify-between border-t border-border bg-card px-4 py-3">
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="default"
            onClick={handleMerge}
            disabled={selected.size < 2}
            leadingIcon={<Merge size={16} />}
          >
            Merge
          </Button>
          <Button
            variant="outline"
            size="default"
            onClick={handleSplitClick}
            disabled={selected.size !== 1}
            leadingIcon={<Split size={16} />}
          >
            Split
          </Button>
          {selected.size > 0 && (
            <span className="text-xs text-muted-foreground">{selected.size} selected</span>
          )}
        </div>

        <Button size="default" onClick={() => onConfirm(tasks)}>
          Confirm &amp; Continue
        </Button>
      </div>
    </div>
  )
}
