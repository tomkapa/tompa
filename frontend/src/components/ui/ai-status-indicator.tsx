import { cn } from '@/lib/utils'
import { StatusBadge } from '@/components/ui/status-badge'

export type AIState = 'running' | 'paused' | 'blocked' | 'done'

interface AIStatusIndicatorProps {
  state: AIState
  statusText: string
  blockedOn?: string
  className?: string
}

type TaskStatusValue = 'running' | 'needs_input' | 'blocked' | 'done'

function toTaskStatus(state: AIState): TaskStatusValue {
  if (state === 'paused') return 'needs_input'
  return state
}

const stateStyles: Record<AIState, { container: string; text: string; subText: string }> = {
  running: {
    container: 'bg-card border-border',
    text: 'text-foreground',
    subText: 'text-muted-foreground',
  },
  paused: {
    container: 'bg-[var(--color-warning)] border-[var(--color-warning)]',
    text: 'text-[var(--color-warning-foreground)] font-medium',
    subText: 'text-[var(--color-warning-foreground)] opacity-80',
  },
  blocked: {
    container: 'bg-[var(--color-error)] border-[var(--color-error)]',
    text: 'text-[var(--color-error-foreground)] font-medium',
    subText: 'text-[var(--color-error-foreground)] opacity-80',
  },
  done: {
    container: 'bg-[var(--color-success)] border-[var(--color-success)]',
    text: 'text-[var(--color-success-foreground)] font-medium',
    subText: 'text-[var(--color-success-foreground)] opacity-80',
  },
}

const stateSubText: Record<AIState, string> = {
  running: '',
  paused: 'Waiting for your answer',
  blocked: 'Waiting on dependency',
  done: 'Ready for review',
}

export function AIStatusIndicator({
  state,
  statusText,
  blockedOn,
  className,
}: AIStatusIndicatorProps) {
  const styles = stateStyles[state]
  const defaultSub = stateSubText[state]

  return (
    <div
      className={cn(
        'flex items-center gap-3 rounded-lg border px-[14px] py-[10px]',
        styles.container,
        className
      )}
    >
      <StatusBadge
        type="task"
        value={toTaskStatus(state)}
        className={cn(state === 'running' && 'animate-pulse')}
      />
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className={cn('truncate text-[13px] leading-snug', styles.text)}>
          {statusText}
        </span>
        {(blockedOn || defaultSub) && (
          <span className={cn('text-[11px] leading-snug', styles.subText)}>
            {blockedOn ?? defaultSub}
          </span>
        )}
      </div>
    </div>
  )
}
