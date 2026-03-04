import * as React from 'react'
import { cn } from '@/lib/utils'
import { DomainTag } from '@/components/ui/domain-tag'
import { RollbackBadge } from '@/components/ui/rollback-badge'
import { AnswerOptionCard } from '@/components/ui/answer-option-card'
import { OtherOption } from '@/components/ui/other-option'
import type { QaQuestion } from './types'

interface QuestionBlockProps {
  question: QaQuestion
  onAnswer: (questionId: string, answerIndex: number | null, answerText: string | null) => void
  isRollbackPoint: boolean
  answered: boolean
  /** When true (superseded round), prevent any reselection */
  locked?: boolean
}

function QuestionBlock({
  question,
  onAnswer,
  isRollbackPoint,
  answered,
  locked = false,
}: QuestionBlockProps) {
  const [otherSelected, setOtherSelected] = React.useState(
    answered && question.answeredIndex == null && question.answeredText != null
  )
  const [otherValue, setOtherValue] = React.useState(question.answeredText ?? '')

  const isAnswered = answered

  function handleSelectOption(idx: number) {
    if (locked) return
    onAnswer(question.id, idx, null)
  }

  function handleSelectOther() {
    if (locked) return
    setOtherSelected(true)
  }

  function handleOtherChange(v: string) {
    setOtherValue(v)
  }

  function handleOtherSubmit() {
    if (locked) return
    if (!otherValue.trim()) return
    onAnswer(question.id, null, otherValue.trim())
  }

  return (
    <div
      className={cn(
        'flex w-full flex-col gap-4 rounded-[6px] border bg-card px-6 py-5 transition-all',
        isRollbackPoint
          ? 'border-[var(--color-info)] border-2'
          : 'border-border'
      )}
    >
      {/* Header */}
      <div className="flex flex-col gap-2.5">
        {/* Tags row */}
        <div className="flex items-center gap-2">
          <DomainTag domain={question.domain} />
          {isRollbackPoint && <RollbackBadge />}
        </div>

        {/* Question text */}
        <p className="text-[15px] font-medium leading-[1.4] text-foreground">
          {question.text}
        </p>

        {/* Rationale */}
        {question.rationale && (
          <p className="text-[13px] italic leading-[1.5] text-muted-foreground">
            {question.rationale}
          </p>
        )}
      </div>

      {/* Answer options */}
      <div className="flex flex-col gap-2">
        {question.options.map((opt, idx) => (
          <AnswerOptionCard
            key={idx}
            label={opt.label}
            pros={opt.pros}
            cons={opt.cons}
            selected={question.answeredIndex === idx}
            recommended={!isAnswered && idx === question.recommendedIndex}
            disabled={locked && question.answeredIndex !== idx}
            onSelect={() => handleSelectOption(idx)}
          />
        ))}

        <OtherOption
          selected={otherSelected || (question.answeredIndex == null && question.answeredText != null)}
          disabled={locked}
          value={otherValue}
          onChange={handleOtherChange}
          onSelect={handleSelectOther}
          onSubmit={handleOtherSubmit}
        />
      </div>
    </div>
  )
}

export { QuestionBlock }
export type { QuestionBlockProps }
