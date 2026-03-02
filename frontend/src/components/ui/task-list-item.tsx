import { cn } from '@/lib/utils'
import { TaskTypeIcon, type TaskType } from './task-type-icon'
import { AttentionDot } from '@/components/ui/attention-dot'
import { StatusBadge } from '@/components/ui/status-badge'

export interface TaskListItemData {
  id: string
  name: string
  taskType: TaskType
  state: string
  needsAttention: boolean
}

interface TaskListItemProps {
  task: TaskListItemData
  onClick: () => void
}

type TaskStatusValue = 'done' | 'running' | 'needs_input' | 'blocked'

function toTaskStatus(state: string): TaskStatusValue {
  if (state === 'done' || state === 'completed') return 'done'
  if (state === 'blocked') return 'blocked'
  if (state === 'paused' || state === 'needs_input') return 'needs_input'
  return 'running'
}

export function TaskListItem({ task, onClick }: TaskListItemProps) {
  const taskStatus = toTaskStatus(task.state)
  const isRunning = taskStatus === 'running'

  return (
    <button
      onClick={onClick}
      className={cn(
        'flex w-full items-center gap-3 rounded-lg border border-border px-3',
        'h-11 text-left transition-colors hover:bg-accent/50',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring'
      )}
    >
      <TaskTypeIcon type={task.taskType} />
      <span className="min-w-0 flex-1 truncate text-sm text-foreground">{task.name}</span>
      {task.needsAttention && <AttentionDot />}
      <StatusBadge
        type="task"
        value={taskStatus}
        className={cn(isRunning && 'animate-pulse')}
      />
    </button>
  )
}
