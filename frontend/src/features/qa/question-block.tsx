import * as React from 'react'
import { ChevronDown, ChevronUp } from 'lucide-react'
import { cn } from '@/lib/utils'
import { IconButton } from '@/components/ui/icon-button'
import { DomainTag } from '@/components/ui/domain-tag'
import { RollbackBadge } from '@/components/ui/rollback-badge'
import { AnswerOptionCard } from '@/components/ui/answer-option-card'
import { OtherOption } from '@/components/ui/other-option'
import { AssigneeAvatar } from './assignee-avatar'
import { useAssignQuestion, useUnassignQuestion } from './use-question-assignment'
import type { QaQuestion } from './types'

interface QuestionBlockProps {
  question: QaQuestion
  roundId: string
  storyId: string
  onAnswer: (questionId: string, answerIndex: number | null, answerText: string | null) => void
  isRollbackPoint: boolean
  answered: boolean
  /** When true (superseded round), prevent any reselection */
  locked?: boolean
}

function QuestionBlock({
  question,
  roundId,
  storyId,
  onAnswer,
  isRollbackPoint,
  answered,
  locked = false,
}: QuestionBlockProps) {
  const [otherSelected, setOtherSelected] = React.useState(
    answered && question.answeredIndex == null && question.answeredText != null
  )
  const [otherValue, setOtherValue] = React.useState(question.answeredText ?? '')
  const [rationaleExpanded, setRationaleExpanded] = React.useState(true)

  const assignMutation = useAssignQuestion(roundId, question.id, storyId)
  const unassignMutation = useUnassignQuestion(roundId, question.id, storyId)

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
      {/* Header: left content + right assignee */}
      <div className="flex items-start justify-between gap-2.5">
        {/* Left: tags + question + rationale */}
        <div className="flex min-w-0 flex-1 flex-col gap-2.5">
          {/* Tags row */}
          <div className="flex items-center gap-2">
            <DomainTag domain={question.domain} />
            {isRollbackPoint && <RollbackBadge />}
          </div>

          {/* Question text + rationale toggle */}
          <div className="flex items-start gap-2">
            <p className="flex-1 text-[15px] font-medium leading-[1.4] text-foreground">
              {question.text}
            </p>
            {question.rationale && (
              <IconButton
                type="button"
                variant="ghost"
                onClick={() => setRationaleExpanded((prev) => !prev)}
                className="mt-0.5 h-6 w-6 shrink-0 text-muted-foreground"
                aria-label={rationaleExpanded ? 'Collapse explanation' : 'Expand explanation'}
              >
                {rationaleExpanded ? (
                  <ChevronUp className="h-4 w-4" />
                ) : (
                  <ChevronDown className="h-4 w-4" />
                )}
              </IconButton>
            )}
          </div>

          {/* Rationale — collapsible */}
          {question.rationale && rationaleExpanded && (
            <p className="text-[13px] italic leading-[1.5] text-muted-foreground">
              {question.rationale}
            </p>
          )}
        </div>

        {/* Right: assignee avatar */}
        <AssigneeAvatar
          assignedTo={question.assignedTo ?? null}
          onAssign={(memberId) => assignMutation.mutate(memberId)}
          onUnassign={() => unassignMutation.mutate()}
          disabled={locked}
        />
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
            dimmed={isAnswered && question.answeredIndex !== idx}
            locked={locked}
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
