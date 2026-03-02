import { cn } from '@/lib/utils'

export interface AttentionDotProps {
  className?: string
}

export function AttentionDot({ className }: AttentionDotProps) {
  return (
    <span
      className={cn(
        'inline-block h-2 w-2 shrink-0 rounded-full bg-destructive animate-pulse',
        className
      )}
      aria-label="Needs attention"
    />
  )
}
