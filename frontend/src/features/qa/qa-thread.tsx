import * as React from 'react'
import { AlertTriangle } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Select } from '@/components/ui/select'
import { NewQuestionIndicator } from '@/components/ui/new-question-indicator'
import { CourseCorrectionInput } from '@/components/ui/course-correction-input'
import { QuestionBlock } from './question-block'
import { PatternIndicatorBadge } from './pattern-indicator-badge'
import type { AppliedPattern, QaRound, QaQuestion } from './types'

// Patterns with override_count > 2 get the "outdated?" prompt per spec §8.
const OVERRIDE_ALERT_THRESHOLD = 2

function OutdatedPatternAlert({ patterns }: { patterns: AppliedPattern[] }) {
  if (patterns.length === 0) return null
  return (
    <div className="flex items-start gap-2 rounded-xl border border-amber-500/30 bg-amber-500/10 px-3.5 py-3">
      <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0 text-amber-500" />
      <div className="flex flex-col gap-0.5">
        <span className="text-[11px] font-medium text-amber-500">
          {patterns.length === 1 ? 'A pattern' : `${patterns.length} patterns`} used here may be outdated
        </span>
        <ul className="flex flex-col gap-0.5">
          {patterns.map((p) => (
            <li key={p.id} className="text-[11px] text-amber-400/80">
              "{p.pattern}" — overridden {p.override_count} time{p.override_count !== 1 ? 's' : ''}
            </li>
          ))}
        </ul>
        <span className="mt-0.5 text-[11px] text-muted-foreground">
          Consider retiring or superseding this pattern in the Patterns page.
        </span>
      </div>
    </div>
  )
}

interface QaThreadProps {
  rounds: QaRound[]
  storyId: string
  stage?: string
  stages?: string[]
  onAnswer: (questionId: string, answerIndex: number | null, answerText: string | null) => void
  onCourseCorrect: (text: string) => void
  onStageChange?: (stage: string) => void
}

function QaThread({
  rounds,
  storyId,
  stage,
  stages,
  onAnswer,
  onCourseCorrect,
  onStageChange,
}: QaThreadProps) {
  const [courseCorrectValue, setCourseCorrectValue] = React.useState('')
  const [showNewIndicator, setShowNewIndicator] = React.useState(false)
  const contentRef = React.useRef<HTMLDivElement>(null)
  const bottomRef = React.useRef<HTMLDivElement>(null)

  // Track scroll position to show indicator when scrolled away from bottom
  React.useEffect(() => {
    const el = contentRef.current
    if (!el) return

    function onScroll() {
      if (!el) return
      const distFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight
      setShowNewIndicator(distFromBottom > 120)
    }

    el.addEventListener('scroll', onScroll)
    return () => el.removeEventListener('scroll', onScroll)
  }, [])

  // Auto-scroll to bottom when new rounds are added
  const prevRoundsLength = React.useRef(rounds.length)
  React.useEffect(() => {
    if (rounds.length > prevRoundsLength.current) {
      const el = contentRef.current
      if (!el) return
      const distFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight
      if (distFromBottom > 120) {
        setShowNewIndicator(true)
      } else {
        bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
      }
    }
    prevRoundsLength.current = rounds.length
  }, [rounds.length])

  function scrollToBottom() {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
    setShowNewIndicator(false)
  }

  function handleCourseCorrectSubmit() {
    if (!courseCorrectValue.trim()) return
    onCourseCorrect(courseCorrectValue.trim())
    setCourseCorrectValue('')
  }

  function isQuestionAnswered(q: QaQuestion): boolean {
    return q.answeredIndex != null || q.answeredText != null
  }

  return (
    <div className="flex h-full flex-col overflow-hidden rounded-[24px] border border-border bg-background">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-5 py-4">
        <h2 className="text-base font-semibold leading-[1.2] text-foreground">Questions</h2>
        {stages && stages.length > 0 && onStageChange && (
          <Select
            value={stage}
            onChange={(e) => onStageChange(e.target.value)}
            options={stages.map((s) => ({ value: s, label: s }))}
            className="w-40"
          />
        )}
      </div>

      {/* Scrollable content */}
      <div className="relative flex-1 overflow-hidden">
        <div ref={contentRef} className="h-full overflow-y-auto">
          <div className="flex flex-col gap-5 p-5">
            {rounds.filter((round) => round.questions.length > 0).map((round) => (
              <React.Fragment key={round.id}>
                {/* Round label divider */}
                <div className="flex items-center gap-2">
                  <div className="h-px flex-1 bg-border" />
                  <span className="text-[11px] font-medium leading-[1.2] text-muted-foreground">
                    Round {round.roundNumber}
                  </span>
                  <div className="h-px flex-1 bg-border" />
                </div>

                {/* Pattern indicator badge */}
                {round.appliedPatternCount != null && round.appliedPatternCount > 0 && (
                  <PatternIndicatorBadge
                    patternCount={round.appliedPatternCount}
                    patterns={round.appliedPatterns}
                  />
                )}

                {/* Outdated pattern alert — shown when injected patterns have high override counts */}
                {round.appliedPatterns && (
                  <OutdatedPatternAlert
                    patterns={round.appliedPatterns.filter(
                      (p) => p.override_count > OVERRIDE_ALERT_THRESHOLD
                    )}
                  />
                )}

                {/* Questions in round */}
                {round.questions.map((q) => (
                  <QuestionBlock
                    key={q.id}
                    question={q}
                    roundId={round.id}
                    storyId={storyId}
                    onAnswer={onAnswer}
                    isRollbackPoint={!!round.isRollbackPoint}
                    answered={isQuestionAnswered(q)}
                    locked={round.status !== 'active'}
                  />
                ))}
              </React.Fragment>
            ))}
            <div ref={bottomRef} />
          </div>
        </div>

        {/* Floating new question indicator */}
        <div
          className={cn(
            'pointer-events-none absolute bottom-4 left-0 right-0 flex justify-center transition-opacity',
            showNewIndicator ? 'pointer-events-auto opacity-100' : 'opacity-0'
          )}
        >
          <NewQuestionIndicator onClick={scrollToBottom} visible={showNewIndicator} />
        </div>
      </div>

      {/* Footer — course correction input */}
      <div className="border-t border-border bg-card px-5 py-4">
        <CourseCorrectionInput
          value={courseCorrectValue}
          onChange={setCourseCorrectValue}
          onSubmit={handleCourseCorrectSubmit}
        />
      </div>
    </div>
  )
}

export { QaThread }
export type { QaThreadProps }
