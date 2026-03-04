import { cn } from '@/lib/utils'

interface DomainTagProps {
  domain: string
  className?: string
}

const DOMAIN_COLORS: Record<string, string> = {
  business:     'border-blue-500/30 bg-blue-500/10 text-blue-400',
  development:  'border-emerald-500/30 bg-emerald-500/10 text-emerald-400',
  backend:      'border-emerald-500/30 bg-emerald-500/10 text-emerald-400',
  design:       'border-purple-500/30 bg-purple-500/10 text-purple-400',
  security:     'border-red-500/30 bg-red-500/10 text-red-400',
  marketing:    'border-amber-500/30 bg-amber-500/10 text-amber-400',
  architecture: 'border-cyan-500/30 bg-cyan-500/10 text-cyan-400',
  planning:     'border-indigo-500/30 bg-indigo-500/10 text-indigo-400',
  infra:        'border-orange-500/30 bg-orange-500/10 text-orange-400',
}

function DomainTag({ domain, className }: DomainTagProps) {
  const colorClass = DOMAIN_COLORS[domain] ?? 'border-border text-muted-foreground'

  return (
    <span
      className={cn(
        'inline-flex items-center justify-center rounded-[6px] border px-2 py-[3px] text-[11px] font-medium leading-[1.2]',
        colorClass,
        className
      )}
    >
      {domain}
    </span>
  )
}

export { DomainTag }
export type { DomainTagProps }
