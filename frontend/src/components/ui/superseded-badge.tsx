import { Undo2 } from 'lucide-react'
import { cn } from '@/lib/utils'

function SupersededBadge({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded bg-muted px-2 py-[3px]',
        'text-[11px] font-medium leading-[1.2] text-muted-foreground',
        className
      )}
    >
      <Undo2 className="h-3 w-3 shrink-0" />
      Superseded
    </span>
  )
}

export { SupersededBadge }
