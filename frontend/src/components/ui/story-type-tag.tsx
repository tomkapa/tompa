import { cn } from '@/lib/utils'

export type StoryType = 'feature' | 'bug' | 'refactor'

export interface StoryTypeTagProps {
  type: StoryType
  className?: string
}

const tagConfig = {
  bug: {
    label: 'BUG',
    className: 'bg-destructive text-destructive-foreground',
  },
  refactor: {
    label: 'REFACTOR',
    className: 'bg-[var(--color-info)] text-[var(--color-info-foreground)]',
  },
} as const

export function StoryTypeTag({ type, className }: StoryTypeTagProps) {
  if (type === 'feature') return null

  const config = tagConfig[type]

  return (
    <span
      className={cn(
        'inline-flex items-center justify-center rounded px-[6px] py-[2px] text-[10px] font-bold leading-[1.2] tracking-[0.5px] whitespace-nowrap shrink-0',
        config.className,
        className
      )}
    >
      {config.label}
    </span>
  )
}
