import { cn } from '@/lib/utils'
import { TaskTypeIcon, type TaskType } from './task-type-icon'
import { AttentionDot } from '@/components/ui/attention-dot'
import { StatusBadge } from '@/components/ui/status-badge'
import { Button } from '@/components/ui/button'

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
    <Button
      variant="ghost"
      onClick={onClick}
      className={cn(
        'w-full justify-start rounded-lg border border-border px-3',
        'h-11 hover:bg-accent/50',
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
    </Button>
  )
}
