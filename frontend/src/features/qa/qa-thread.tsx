import * as React from 'react'
import { cn } from '@/lib/utils'
import { NewQuestionIndicator } from '@/components/ui/new-question-indicator'
import { CourseCorrectionInput } from '@/components/ui/course-correction-input'
import { QuestionBlock } from './question-block'
import type { QaRound, QaQuestion } from './types'

interface QaThreadProps {
  rounds: QaRound[]
  stage?: string
  stages?: string[]
  onAnswer: (questionId: string, answerIndex: number | null, answerText: string | null) => void
  onRollback?: (roundId: string) => void
  onCourseCorrect: (text: string) => void
  onStageChange?: (stage: string) => void
}

function QaThread({
  rounds,
  stage,
  stages,
  onAnswer,
  onRollback,
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
          <select
            value={stage}
            onChange={(e) => onStageChange(e.target.value)}
            className="rounded-full border border-border bg-background px-3 py-1.5 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
          >
            {stages.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        )}
      </div>

      {/* Scrollable content */}
      <div ref={contentRef} className="relative flex-1 overflow-y-auto">
        <div className="flex flex-col gap-5 p-5">
          {rounds.map((round) => (
            <React.Fragment key={round.id}>
              {/* Round label divider */}
              <div className="flex items-center gap-2">
                <div className="h-px flex-1 bg-border" />
                <span className="text-[11px] font-medium leading-[1.2] text-muted-foreground">
                  Round {round.roundNumber}
                </span>
                <div className="h-px flex-1 bg-border" />
              </div>

              {/* Questions in round */}
              {round.questions.map((q) => (
                <QuestionBlock
                  key={q.id}
                  question={q}
                  onAnswer={onAnswer}
                  onRollback={onRollback ? () => onRollback(round.id) : undefined}
                  isRollbackPoint={!!round.isRollbackPoint}
                  answered={isQuestionAnswered(q)}
                />
              ))}
            </React.Fragment>
          ))}
          <div ref={bottomRef} />
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
