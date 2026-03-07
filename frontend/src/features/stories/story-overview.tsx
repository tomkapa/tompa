import { cn } from '@/lib/utils'
import type { StoryResponse, TaskSummary } from '@/api/generated/tompaAPI.schemas'
import { TaskListItem } from '@/components/ui/task-list-item'
import { StatusBadge } from '@/components/ui/status-badge'
import { Button } from '@/components/ui/button'
import type { TaskType } from '@/components/ui/task-type-icon'
import { MarkdownEditor } from '@/components/ui/markdown-editor'
import { ExpandableMarkdownViewer } from '@/components/ui/markdown-viewer'

interface StoryOverviewProps {
  story: StoryResponse
  tasks: TaskSummary[]
  onTaskClick: (taskId: string) => void
  onApproveDescription?: () => void
  onDescriptionSave?: (description: string) => void
  isSavingDescription?: boolean
  className?: string
}

const KNOWN_TASK_TYPES = new Set<string>(['design', 'test', 'code'])

function toTaskType(raw: string): TaskType {
  return KNOWN_TASK_TYPES.has(raw) ? (raw as TaskType) : 'code'
}

type StoryStatusValue = 'todo' | 'in_progress' | 'done'

function toStoryStatus(status: string): StoryStatusValue {
  if (status === 'in_progress') return 'in_progress'
  if (status === 'done') return 'done'
  return 'todo'
}

export function StoryOverview({ story, tasks, onTaskClick, onApproveDescription, onDescriptionSave, isSavingDescription, className }: StoryOverviewProps) {
  return (
    <div
      className={cn(
        'flex flex-col overflow-hidden rounded-2xl border border-border bg-background',
        className
      )}
    >
      {/* Panel header */}
      <div className="flex shrink-0 flex-col gap-3 border-b border-border px-5 py-4">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-base font-semibold leading-snug text-foreground">{story.title}</h2>
          <div className="flex shrink-0 items-center gap-3">
            <StatusBadge type="story" value={toStoryStatus(story.status)} />
          </div>
        </div>
        <div className="max-h-52 overflow-y-auto">
          <MarkdownEditor
            value={story.description ?? ''}
            onSave={onDescriptionSave ?? (() => {})}
            isSaving={isSavingDescription}
            readOnly={!onDescriptionSave}
            placeholder="No description — double-click to add one."
          />
        </div>
      </div>

      {/* Pending refined description */}
      {story.pending_refined_description && (
        <div className="flex shrink-0 flex-col gap-3 border-b border-border bg-accent/40 px-5 py-4">
          <div className="flex items-center justify-between gap-2">
            <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
              AI-refined description
            </span>
            {onApproveDescription && (
              <Button size="default" variant="default" onClick={onApproveDescription}>
                Approve
              </Button>
            )}
          </div>
          <div className="max-h-72 overflow-y-auto">
            <ExpandableMarkdownViewer
              content={story.pending_refined_description ?? ''}
              label="AI-refined description"
            />
          </div>
        </div>
      )}

      {/* Task list section */}
      <div className="flex flex-1 flex-col overflow-hidden">
        <div className="flex items-center justify-between border-b border-border px-5 py-3">
          <span className="text-sm font-semibold text-foreground">Tasks</span>
          <span className="text-xs text-muted-foreground">{tasks.length} tasks</span>
        </div>

        <div className="flex flex-col overflow-y-auto">
          {tasks.map((task) => (
            <TaskListItem
              key={task.id}
              task={{
                id: task.id,
                name: task.name,
                taskType: toTaskType(task.task_type),
                state: task.state,
                needsAttention: task.state === 'paused' || task.state === 'needs_input',
              }}
              onClick={() => onTaskClick(task.id)}
            />
          ))}
        </div>
      </div>
    </div>
  )
}
