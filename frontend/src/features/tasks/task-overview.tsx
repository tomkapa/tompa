import { cn } from '@/lib/utils'
import type { TaskResponse } from '@/api/generated/tompaAPI.schemas'
import { TaskTypeIcon, type TaskType } from '@/components/ui/task-type-icon'
import { AIStatusIndicator, type AIState } from '@/components/ui/ai-status-indicator'
import { MarkDoneButton } from '@/components/ui/mark-done-button'

interface TaskOverviewProps {
  task: TaskResponse
  onMarkDone: () => void
  markDoneLoading?: boolean
  className?: string
}

const KNOWN_TASK_TYPES = new Set<string>(['design', 'test', 'code'])

function toTaskType(raw: string): TaskType {
  return KNOWN_TASK_TYPES.has(raw) ? (raw as TaskType) : 'code'
}

function toAIState(state: string): AIState {
  if (state === 'done' || state === 'completed') return 'done'
  if (state === 'paused') return 'paused'
  if (state === 'blocked') return 'blocked'
  return 'running'
}

export function TaskOverview({ task, onMarkDone, markDoneLoading, className }: TaskOverviewProps) {
  const aiState = toAIState(task.state)
  const isDone = aiState === 'done'

  return (
    <div
      className={cn(
        'flex flex-col overflow-hidden rounded-2xl border border-border bg-background',
        className
      )}
    >
      {/* Task header */}
      <div className="flex flex-col gap-4 border-b border-border px-5 py-4">
        <div className="flex w-full items-center gap-3">
          <TaskTypeIcon type={toTaskType(task.task_type)} />
          <h2 className="text-base font-semibold leading-snug text-foreground">{task.name}</h2>
        </div>

        <p className="text-[13px] leading-relaxed text-muted-foreground">{task.description}</p>

        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">Assigned to</span>
          <span className="text-[13px] font-medium text-foreground">AI Agent</span>
        </div>
      </div>

      {/* Task content */}
      <div className="flex flex-col gap-4 p-5">
        <span className="text-xs font-medium text-muted-foreground">AI Status</span>

        <AIStatusIndicator
          state={aiState}
          statusText={task.ai_status_text ?? 'Working on task…'}
          className="w-full"
        />

        {isDone && (
          <MarkDoneButton onClick={onMarkDone} loading={markDoneLoading} />
        )}
      </div>
    </div>
  )
}
