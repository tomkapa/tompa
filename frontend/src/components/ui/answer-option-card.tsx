import * as React from 'react'
import { ChevronDown, ChevronUp, ThumbsUp, ThumbsDown, Sparkles } from 'lucide-react'
import { cn } from '@/lib/utils'

interface AnswerOptionCardProps {
  label: string
  pros: string
  cons: string
  selected: boolean
  recommended: boolean
  disabled: boolean
  onSelect: () => void
}

function AnswerOptionCard({
  label,
  pros,
  cons,
  selected,
  recommended,
  disabled,
  onSelect,
}: AnswerOptionCardProps) {
  const [expanded, setExpanded] = React.useState(false)

  function handleRadioClick(e: React.MouseEvent) {
    e.stopPropagation()
    if (disabled && !selected) return
    onSelect()
  }

  function handleCardClick() {
    if (disabled && !selected) return
    if (selected) return
    setExpanded((prev) => !prev)
  }

  // Selected state: primary bg, white text, no chevron, no pros/cons
  if (selected) {
    return (
      <button
        type="button"
        disabled
        className="flex w-full items-center gap-3 rounded-[var(--radius-xs)] border border-primary bg-primary px-4 py-[14px] text-left pointer-events-none"
      >
        {/* Radio selected */}
        <span className="flex h-4 w-4 shrink-0 items-center justify-center rounded-full border-2 border-primary-foreground">
          <span className="h-2 w-2 rounded-full bg-primary-foreground" />
        </span>
        <span className="flex-1 text-sm font-medium leading-[1.4] text-primary-foreground">
          {label}
        </span>
      </button>
    )
  }

  // Disabled (answered, not selected): opacity 0.5, no interaction
  if (disabled) {
    return (
      <div className="flex w-full flex-col rounded-[var(--radius-xs)] border border-border bg-background opacity-50 pointer-events-none">
        <div className="flex items-center gap-3 px-4 py-[14px]">
          {/* Radio empty */}
          <span className="flex h-4 w-4 shrink-0 items-center justify-center rounded-full border-2 border-border bg-background" />
          <span className="flex-1 text-sm font-normal leading-[1.4] text-foreground">
            {label}
          </span>
          <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
        </div>
      </div>
    )
  }

  // Default / Expanded / AI Recommended states
  return (
    <div
      className={cn(
        'flex w-full flex-col rounded-[var(--radius-xs)] border bg-background transition-all duration-150 cursor-pointer',
        recommended ? 'border-primary' : 'border-border'
      )}
      onClick={handleCardClick}
    >
      {/* Card Header */}
      <div className="flex items-center gap-3 px-4 py-[14px]">
        {/* Radio empty — clicking selects the option */}
        <button
          type="button"
          onClick={handleRadioClick}
          className="flex h-4 w-4 shrink-0 items-center justify-center rounded-full border-2 border-border bg-background hover:border-primary transition-colors"
          aria-label={`Select ${label}`}
        />

        {/* Text column */}
        {recommended ? (
          <div className="flex flex-1 flex-col gap-1">
            <span className="text-sm font-normal leading-[1.4] text-foreground">
              {label}
            </span>
            <span className="flex items-center gap-1">
              <Sparkles className="h-3 w-3 text-primary" />
              <span className="text-[11px] font-medium leading-[1.2] text-primary">
                AI suggested
              </span>
            </span>
          </div>
        ) : (
          <span className="flex-1 text-sm font-normal leading-[1.4] text-foreground">
            {label}
          </span>
        )}

        {/* Chevron indicator */}
        <span className="shrink-0 p-0.5 text-muted-foreground">
          {expanded ? (
            <ChevronUp className="h-4 w-4" />
          ) : (
            <ChevronDown className="h-4 w-4" />
          )}
        </span>
      </div>

      {/* Pros/Cons Section (expanded) */}
      {expanded && (
        <div className="flex flex-col gap-3 px-4 pb-[14px] pl-[44px] pr-4">
          {/* Pros */}
          <div className="flex gap-2">
            <ThumbsUp className="mt-0.5 h-3.5 w-3.5 shrink-0 text-[var(--color-success-foreground)]" />
            <p className="text-[13px] font-normal leading-[1.5] text-muted-foreground">
              {pros}
            </p>
          </div>
          {/* Cons */}
          <div className="flex gap-2">
            <ThumbsDown className="mt-0.5 h-3.5 w-3.5 shrink-0 text-[var(--color-error-foreground)]" />
            <p className="text-[13px] font-normal leading-[1.5] text-muted-foreground">
              {cons}
            </p>
          </div>
        </div>
      )}
    </div>
  )
}

export { AnswerOptionCard }
export type { AnswerOptionCardProps }
