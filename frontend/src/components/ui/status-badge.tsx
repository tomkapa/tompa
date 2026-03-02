import { cn } from '@/lib/utils'

type StoryStatus = 'todo' | 'in_progress' | 'done'
type TaskStatus = 'done' | 'running' | 'needs_input' | 'blocked'

interface StatusBadgeStoryProps {
  type: 'story'
  value: StoryStatus
  className?: string
}

interface StatusBadgeTaskProps {
  type: 'task'
  value: TaskStatus
  className?: string
}

export type StatusBadgeProps = StatusBadgeStoryProps | StatusBadgeTaskProps

const storyConfig: Record<StoryStatus, { label: string; className: string }> = {
  todo: {
    label: 'To Do',
    className: 'bg-secondary text-secondary-foreground',
  },
  in_progress: {
    label: 'In Progress',
    className: 'bg-[var(--color-info)] text-[var(--color-info-foreground)]',
  },
  done: {
    label: 'Done',
    className: 'bg-[var(--color-success)] text-[var(--color-success-foreground)]',
  },
}

const taskConfig: Record<TaskStatus, { label: string; className: string }> = {
  done: {
    label: 'Done',
    className: 'bg-[var(--color-success)] text-[var(--color-success-foreground)]',
  },
  running: {
    label: 'AI working',
    className: 'bg-primary text-primary-foreground',
  },
  needs_input: {
    label: 'Needs input',
    className: 'bg-[var(--color-warning)] text-[var(--color-warning-foreground)]',
  },
  blocked: {
    label: 'Blocked',
    className: 'bg-[var(--color-error)] text-[var(--color-error-foreground)]',
  },
}

export function StatusBadge({ type, value, className }: StatusBadgeProps) {
  const config =
    type === 'story'
      ? storyConfig[value as StoryStatus]
      : taskConfig[value as TaskStatus]

  return (
    <span
      className={cn(
        'inline-flex items-center justify-center rounded-full px-[10px] py-1 text-xs font-medium leading-[1.2] whitespace-nowrap',
        config.className,
        className
      )}
    >
      {config.label}
    </span>
  )
}
