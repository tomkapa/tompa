import * as React from 'react'
import { CornerDownRight, Lightbulb, ChevronDown, ChevronUp } from 'lucide-react'
import { cn } from '@/lib/utils'
import { DomainTag } from './domain-tag'
import { SupersededBadge } from './superseded-badge'
import { Avatar } from './avatar'
import { Badge } from './badge'
import { Button } from './button'

interface InfluencingPattern {
  id: string
  domain: string
  pattern: string
}

interface DecisionTrailEntryProps {
  domain: string
  questionText: string
  answerText: string
  superseded: boolean
  entryUrl?: string
  className?: string
  answerer?: { displayName: string; avatarUrl?: string | null }
  influencedByPatterns?: InfluencingPattern[]
}

function getInitials(name: string): string {
  return name
    .split(' ')
    .map((w) => w[0])
    .join('')
    .slice(0, 2)
    .toUpperCase()
}

function DecisionTrailEntry({
  domain,
  questionText,
  answerText,
  superseded,
  entryUrl,
  className,
  answerer,
  influencedByPatterns,
}: DecisionTrailEntryProps) {
  const [patternsExpanded, setPatternsExpanded] = React.useState(false)
  const containerClass = cn(
    'flex flex-col gap-2 rounded-[6px] border border-border px-4 py-3',
    superseded ? 'bg-muted opacity-70' : 'bg-card',
    className
  )

  const answererAvatar = answerer ? (
    <Avatar
      src={answerer.avatarUrl ?? undefined}
      initials={getInitials(answerer.displayName)}
      size="sm"
      className="h-6 w-6 shrink-0 text-[9px]"
      title={answerer.displayName}
    />
  ) : null

  const body = superseded ? (
    <>
      <div className="flex items-center gap-2">
        <DomainTag domain={domain} />
        <SupersededBadge />
        {answererAvatar && <div className="ml-auto">{answererAvatar}</div>}
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
        {answererAvatar}
      </div>
      <div className="flex gap-2">
        <CornerDownRight className="mt-[1px] h-[14px] w-[14px] shrink-0 text-muted-foreground" />
        <p className="text-[13px] font-normal leading-[1.3] text-muted-foreground">{answerText}</p>
      </div>
    </>
  )

  const patternFooter =
    influencedByPatterns && influencedByPatterns.length > 0 ? (
      <div className="flex flex-col gap-1 pt-1">
        <Button
          type="button"
          variant="ghost"
          className="h-auto w-fit gap-1 px-0 py-0 text-[11px] font-medium text-purple-400 hover:bg-transparent hover:text-purple-300"
          onClick={(e) => {
            e.preventDefault()
            setPatternsExpanded((v) => !v)
          }}
        >
          <Lightbulb className="h-3 w-3 shrink-0" />
          Influenced by {influencedByPatterns.length} pattern
          {influencedByPatterns.length !== 1 ? 's' : ''}
          {patternsExpanded ? (
            <ChevronUp className="h-3 w-3" />
          ) : (
            <ChevronDown className="h-3 w-3" />
          )}
        </Button>
        {patternsExpanded && (
          <div className="ml-1 flex flex-col gap-1 border-l-2 border-purple-500/20 pl-3">
            {influencedByPatterns.map((p) => (
              <div key={p.id} className="flex items-start gap-1.5">
                <Badge variant="default" className="shrink-0 text-[10px] font-normal capitalize">
                  {p.domain}
                </Badge>
                <span className="text-[11px] leading-relaxed text-muted-foreground">
                  {p.pattern}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    ) : null

  if (entryUrl) {
    return (
      <a href={entryUrl} className={containerClass}>
        {body}
        {patternFooter}
      </a>
    )
  }

  return (
    <div className={containerClass}>
      {body}
      {patternFooter}
    </div>
  )
}

export { DecisionTrailEntry }
export type { DecisionTrailEntryProps, InfluencingPattern }
