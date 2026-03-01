import * as React from 'react'
import { cn } from '@/lib/utils'

// Halo Progress — horizontal bar, matches Progress component
// Track: bg-secondary, rounded-full
// Fill: bg-primary, rounded-full
interface ProgressProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: number // 0–100
  max?: number
}

function Progress({ value = 0, max = 100, className, ...props }: ProgressProps) {
  const pct = Math.max(0, Math.min(100, (value / max) * 100))

  return (
    <div
      role="progressbar"
      aria-valuenow={value}
      aria-valuemin={0}
      aria-valuemax={max}
      className={cn('relative h-4 w-full overflow-hidden rounded-full bg-secondary', className)}
      {...props}
    >
      <div
        className="h-full rounded-full bg-primary transition-all duration-300"
        style={{ width: `${pct}%` }}
      />
    </div>
  )
}

export { Progress }
