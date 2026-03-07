import { CornerDownRight } from 'lucide-react'
import { cn } from '@/lib/utils'
import { DomainTag } from './domain-tag'
import { SupersededBadge } from './superseded-badge'
import { Avatar } from './avatar'

interface DecisionTrailEntryProps {
  domain: string
  questionText: string
  answerText: string
  superseded: boolean
  entryUrl?: string
  className?: string
  answerer?: { displayName: string; avatarUrl?: string | null }
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
}: DecisionTrailEntryProps) {
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
