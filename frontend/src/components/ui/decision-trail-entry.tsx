import { CornerDownRight } from 'lucide-react'
import { cn } from '@/lib/utils'
import { DomainTag } from './domain-tag'
import { SupersededBadge } from './superseded-badge'

interface DecisionTrailEntryProps {
  domain: string
  questionText: string
  answerText: string
  superseded: boolean
  entryUrl?: string
  className?: string
}

function DecisionTrailEntry({
  domain,
  questionText,
  answerText,
  superseded,
  entryUrl,
  className,
}: DecisionTrailEntryProps) {
  const containerClass = cn(
    'flex flex-col gap-2 rounded-[6px] border border-border px-4 py-3',
    superseded ? 'bg-muted opacity-70' : 'bg-card',
    className
  )

  const body = superseded ? (
    <>
      <div className="flex items-center gap-2">
        <DomainTag domain={domain} />
        <SupersededBadge />
      </div>
      <p className="text-[13px] font-medium leading-[1.3] text-muted-foreground">
        {questionText}
      </p>
      <div className="flex gap-2">
        <CornerDownRight className="mt-[1px] h-[14px] w-[14px] shrink-0 text-muted-foreground" />
        <p className="text-[13px] font-normal leading-[1.3] text-muted-foreground">{answerText}</p>
      </div>
    </>
  ) : (
    <>
      <div className="flex w-full items-center gap-2">
        <DomainTag domain={domain} />
        <p className="min-w-0 flex-1 text-[13px] font-medium leading-[1.3] text-foreground">
          {questionText}
        </p>
      </div>
      <div className="flex gap-2">
        <CornerDownRight className="mt-[1px] h-[14px] w-[14px] shrink-0 text-muted-foreground" />
        <p className="text-[13px] font-normal leading-[1.3] text-muted-foreground">{answerText}</p>
      </div>
    </>
  )

  if (entryUrl) {
    return (
      <a href={entryUrl} className={containerClass}>
        {body}
      </a>
    )
  }

  return <div className={containerClass}>{body}</div>
}

export { DecisionTrailEntry }
export type { DecisionTrailEntryProps }
