import { cn } from '@/lib/utils'

interface DomainTagProps {
  domain: string
  className?: string
}

function DomainTag({ domain, className }: DomainTagProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center justify-center rounded-[6px] border border-border px-2 py-[3px] text-[11px] font-medium leading-[1.2] text-muted-foreground',
        className
      )}
    >
      {domain}
    </span>
  )
}

export { DomainTag }
export type { DomainTagProps }
