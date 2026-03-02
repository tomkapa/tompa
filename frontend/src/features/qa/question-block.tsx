import * as React from 'react'
import { Undo2 } from 'lucide-react'
import { cn } from '@/lib/utils'
import { DomainTag } from '@/components/ui/domain-tag'
import { RollbackBadge } from '@/components/ui/rollback-badge'
import { AnswerOptionCard } from '@/components/ui/answer-option-card'
import { OtherOption } from '@/components/ui/other-option'
import type { QaQuestion } from './types'

interface QuestionBlockProps {
  question: QaQuestion
  onAnswer: (questionId: string, answerIndex: number | null, answerText: string | null) => void
  onRollback?: () => void
  isRollbackPoint: boolean
  answered: boolean
}

function QuestionBlock({
  question,
  onAnswer,
  onRollback,
  isRollbackPoint,
  answered,
}: QuestionBlockProps) {
  const [isHovered, setIsHovered] = React.useState(false)
  const [otherSelected, setOtherSelected] = React.useState(
    answered && question.answeredIndex == null && question.answeredText != null
  )
  const [otherValue, setOtherValue] = React.useState(question.answeredText ?? '')

  const isAnswered = answered

  function handleSelectOption(idx: number) {
    if (isAnswered) return
    onAnswer(question.id, idx, null)
  }

  function handleSelectOther() {
    if (isAnswered) return
    setOtherSelected(true)
  }

  function handleOtherChange(v: string) {
    setOtherValue(v)
  }

  function handleOtherSubmit() {
    if (!otherValue.trim()) return
    onAnswer(question.id, null, otherValue.trim())
  }

  function handleUndo() {
    onRollback?.()
    setOtherSelected(false)
    setOtherValue('')
  }

  return (
    <div
      className={cn(
        'flex w-full flex-col gap-4 rounded-[6px] border bg-card px-6 py-5 transition-all',
        isRollbackPoint
          ? 'border-[var(--color-info)] border-2'
          : 'border-border',
        isAnswered && isHovered && 'cursor-default'
      )}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Header */}
      <div className="flex flex-col gap-2.5">
        {/* Tags row */}
        <div className="flex items-center gap-2">
          <DomainTag domain={question.domain} />
          {isRollbackPoint && <RollbackBadge />}
        </div>

        {/* Question text + optional undo button */}
        <div className={cn('flex items-start gap-2.5', isAnswered && 'justify-between')}>
          <p className="flex-1 text-[15px] font-medium leading-[1.4] text-foreground">
            {question.text}
          </p>
          {isAnswered && isHovered && (
            <button
              type="button"
              onClick={handleUndo}
              className="flex shrink-0 items-center gap-1.5 rounded-[6px] border border-border bg-muted px-2.5 py-1.5 text-xs font-medium text-foreground transition-colors hover:bg-accent"
            >
              <Undo2 className="h-3.5 w-3.5" strokeWidth={2} />
              Undo
            </button>
          )}
        </div>
      </div>

      {/* Answer options */}
      <div className="flex flex-col gap-2">
        {question.options.map((opt, idx) => (
          <AnswerOptionCard
            key={idx}
            text={opt.text}
            selected={isAnswered && question.answeredIndex === idx}
            disabled={isAnswered && question.answeredIndex !== idx}
            onSelect={() => handleSelectOption(idx)}
          />
        ))}

        <OtherOption
          selected={otherSelected || (isAnswered && question.answeredIndex == null && question.answeredText != null)}
          disabled={isAnswered && !(question.answeredIndex == null && question.answeredText != null)}
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
