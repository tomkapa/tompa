import { Flag } from 'lucide-react'
import { cn } from '@/lib/utils'

interface RollbackBadgeProps {
  className?: string
}

function RollbackBadge({ className }: RollbackBadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-[4px] px-2 py-[3px]',
        'bg-[var(--color-info)] text-[var(--color-info-foreground)]',
        className
      )}
    >
      <Flag className="h-3 w-3" strokeWidth={2} />
      <span className="text-[11px] font-medium leading-[1.2]">Rollback Point</span>
    </span>
  )
}

export { RollbackBadge }
