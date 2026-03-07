import { cn } from '@/lib/utils'
import { Progress } from '@/components/ui/progress'

interface ConfidenceBarProps {
  confidence: number
  className?: string
}

function confidenceColor(confidence: number): string {
  if (confidence >= 0.7) return 'text-emerald-400'
  if (confidence >= 0.4) return 'text-amber-400'
  return 'text-red-400'
}

function confidenceTrackColor(confidence: number): string {
  if (confidence >= 0.7) return '[&>div]:bg-emerald-500'
  if (confidence >= 0.4) return '[&>div]:bg-amber-500'
  return '[&>div]:bg-red-500'
}

function ConfidenceBar({ confidence, className }: ConfidenceBarProps) {
  const pct = Math.round(confidence * 100)

  return (
    <div className={cn('flex items-center gap-2', className)}>
      <Progress
        value={pct}
        max={100}
        className={cn('h-2 w-20', confidenceTrackColor(confidence))}
      />
      <span className={cn('text-xs font-medium tabular-nums', confidenceColor(confidence))}>
        {pct}%
      </span>
    </div>
  )
}

export { ConfidenceBar }
