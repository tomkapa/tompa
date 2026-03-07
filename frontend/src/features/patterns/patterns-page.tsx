import * as React from 'react'
import { Lightbulb } from 'lucide-react'
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
      <div className="flex shrink-0 items-center justify-between">
        <h1 className="text-xl font-semibold leading-none text-foreground md:text-2xl">
          Decision Patterns
        </h1>
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
            <span className="text-sm text-muted-foreground">Loading patterns...</span>
          </div>
        ) : patterns.length === 0 ? (
          <EmptyState />
        ) : (
          <div className="flex flex-col gap-3">
            {patterns.map((p) => (
              <PatternRow
                key={p.id}
                pattern={p}
                onClick={() => setSelectedPattern(p)}
              />
            ))}
          </div>
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
      className="cursor-pointer transition-shadow hover:shadow-md rounded-2xl"
      onClick={onClick}
    >
      <CardContent className="flex items-center gap-4 py-4">
        {/* Domain badge */}
        <DomainTag domain={pattern.domain} className="shrink-0" />

        {/* Pattern text */}
        <p className="min-w-0 flex-1 truncate text-sm text-foreground">
          {pattern.pattern}
        </p>

        {/* Confidence bar */}
        <ConfidenceBar confidence={pattern.confidence} className="shrink-0" />

        {/* Usage count */}
        <span className="shrink-0 text-xs tabular-nums text-muted-foreground">
          {pattern.usage_count} uses
        </span>
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
