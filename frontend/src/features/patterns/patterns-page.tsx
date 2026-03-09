import * as React from 'react'
import { Lightbulb, Brain } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import { DomainTag } from '@/components/ui/domain-tag'
import { ConfidenceBar } from './confidence-bar'
import { PatternFilters } from './pattern-filters'
import { PatternDetail } from './pattern-detail'
import { useListPatterns } from './use-patterns'
import type { DecisionPatternResponse, PatternDomain } from './types'

interface PatternsPageProps {
  projectId: string
}

function PatternsPage({ projectId }: PatternsPageProps) {
  const [domain, setDomain] = React.useState<PatternDomain | ''>('')
  const [minConfidence, setMinConfidence] = React.useState(0)
  const [selectedPattern, setSelectedPattern] = React.useState<DecisionPatternResponse | null>(null)

  const params = React.useMemo(() => ({
    ...(domain ? { domain } : {}),
    ...(minConfidence > 0 ? { min_confidence: minConfidence } : {}),
  }), [domain, minConfidence])

  const { data: resp, isLoading, error } = useListPatterns(
    projectId,
    params,
    { fetch: { credentials: 'include' } },
  )

  const patterns = resp?.status === 200 ? resp.data : []

  if (error) {
    console.error('[PatternsPage] load failed', { projectId }, error)
  }

  return (
    <div className="flex h-full flex-col gap-6 overflow-hidden">
      {/* Header */}
      <div className="shrink-0 space-y-1">
        <div className="flex items-center gap-2">
          <Brain className="h-5 w-5 text-muted-foreground" />
          <h1 className="text-xl font-semibold leading-none text-foreground md:text-2xl">
            Decision Patterns
          </h1>
        </div>
        <p className="pl-7 text-sm text-muted-foreground">
          Recurring architectural choices extracted from your Q&amp;A rounds
        </p>
      </div>

      {/* Filters */}
      <div className="shrink-0">
        <PatternFilters
          domain={domain}
          minConfidence={minConfidence}
          onDomainChange={setDomain}
          onMinConfidenceChange={setMinConfidence}
        />
      </div>

      {/* Pattern list */}
      <div className="flex-1 overflow-y-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-20">
            <span className="text-sm text-muted-foreground">Loading patterns…</span>
          </div>
        ) : patterns.length === 0 ? (
          <EmptyState />
        ) : (
          <>
            <p className="mb-3 text-xs text-muted-foreground">
              {patterns.length} {patterns.length === 1 ? 'pattern' : 'patterns'}
            </p>
            <div className="flex flex-col gap-2">
              {patterns.map((p) => (
                <PatternRow
                  key={p.id}
                  pattern={p}
                  onClick={() => setSelectedPattern(p)}
                />
              ))}
            </div>
          </>
        )}
      </div>

      {/* Detail drawer */}
      {selectedPattern && (
        <PatternDetail
          pattern={selectedPattern}
          projectId={projectId}
          onClose={() => setSelectedPattern(null)}
        />
      )}
    </div>
  )
}

function PatternRow({ pattern, onClick }: { pattern: DecisionPatternResponse; onClick: () => void }) {
  return (
    <Card
      className="cursor-pointer rounded-xl border-border/50 bg-card/60 transition-all hover:border-border hover:bg-card hover:shadow-sm"
      onClick={onClick}
    >
      <CardContent className="flex flex-col gap-2 px-4 py-3">
        {/* Top row: domain + confidence + uses */}
        <div className="flex items-center justify-between gap-3">
          <DomainTag domain={pattern.domain} />
          <div className="flex items-center gap-3">
            <ConfidenceBar confidence={pattern.confidence} />
            <span className="text-xs tabular-nums text-muted-foreground">
              {pattern.usage_count} {pattern.usage_count === 1 ? 'use' : 'uses'}
            </span>
          </div>
        </div>

        {/* Pattern text */}
        <p className="line-clamp-2 text-sm leading-relaxed text-foreground/90">
          {pattern.pattern}
        </p>
      </CardContent>
    </Card>
  )
}

function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-20 text-center">
      <div className="flex h-12 w-12 items-center justify-center rounded-full bg-accent">
        <Lightbulb className="h-6 w-6 text-muted-foreground" />
      </div>
      <p className="text-sm font-medium text-foreground">No patterns yet</p>
      <p className="max-w-sm text-sm text-muted-foreground">
        Decision patterns are automatically extracted from your Q&A rounds. Complete a few stories
        and patterns will appear here.
      </p>
    </div>
  )
}

export { PatternsPage }
