import * as React from 'react'
import { Lightbulb, ChevronDown, ChevronUp } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import type { AppliedPattern } from './types'

interface PatternIndicatorBadgeProps {
  patternCount: number
  patterns?: AppliedPattern[]
  className?: string
}

function PatternIndicatorBadge({ patternCount, patterns, className }: PatternIndicatorBadgeProps) {
  const [expanded, setExpanded] = React.useState(false)

  if (patternCount <= 0) return null

  const hasPatterns = patterns && patterns.length > 0

  return (
    <div className={cn('flex flex-col gap-1', className)}>
      <Button
        type="button"
        variant="ghost"
        className="h-auto gap-1.5 rounded-full border border-purple-500/30 bg-purple-500/10 px-2.5 py-1 text-[11px] font-medium text-purple-400 hover:bg-purple-500/20 hover:text-purple-400"
        onClick={() => hasPatterns && setExpanded(!expanded)}
        aria-expanded={expanded}
      >
        <Lightbulb className="h-3 w-3" />
        Based on {patternCount} project pattern{patternCount !== 1 ? 's' : ''}
        {hasPatterns && (expanded ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />)}
      </Button>

      {expanded && hasPatterns && (
        <div className="ml-1 flex flex-col gap-1.5 border-l-2 border-purple-500/20 pl-3">
          {patterns.map((p) => (
            <div key={p.id} className="flex items-start gap-1.5">
              <Badge variant="default" className="shrink-0 text-[10px] font-normal capitalize">
                {p.domain}
              </Badge>
              <span className="text-[11px] leading-relaxed text-muted-foreground">{p.pattern}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export { PatternIndicatorBadge }
export type { PatternIndicatorBadgeProps }
