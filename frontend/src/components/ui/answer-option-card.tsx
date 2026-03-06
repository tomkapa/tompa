import * as React from 'react'
import { ChevronDown, ChevronUp, ThumbsUp, ThumbsDown, Sparkles } from 'lucide-react'
import { cn } from '@/lib/utils'

interface AnswerOptionCardProps {
  label: string
  pros: string
  cons: string
  selected: boolean
  recommended: boolean
  /** Dims non-selected options when another option in the same question is already chosen */
  dimmed: boolean
  /** Completely prevents selection (historical/superseded round) */
  locked: boolean
  onSelect: () => void
}

function AnswerOptionCard({
  label,
  pros,
  cons,
  selected,
  recommended,
  dimmed,
  locked,
  onSelect,
}: AnswerOptionCardProps) {
  const [expanded, setExpanded] = React.useState(false)

  function handleRadioClick(e: React.MouseEvent) {
    e.stopPropagation()
    if (selected) return
    onSelect()
  }

  function handleCardClick() {
    setExpanded((prev) => !prev)
  }

  // Locked + not selected: completely inert — no click, no hover, no expand
  if (locked && !selected) {
    return (
      <div className="flex w-full flex-col rounded-[var(--radius-xs)] border border-border bg-background opacity-50 pointer-events-none">
        <div className="flex items-center gap-3 px-4 py-[14px]">
          <span className="flex h-4 w-4 shrink-0 items-center justify-center rounded-full border-2 border-border bg-background" />
          <span className="flex-1 text-sm font-normal leading-[1.4] text-foreground">{label}</span>
        </div>
      </div>
    )
  }

  // All other states: selected (any round), or active-round unselected
  return (
    <div
      className={cn(
        'flex w-full flex-col rounded-[var(--radius-xs)] border transition-all duration-150 cursor-pointer',
        selected
          ? 'border-primary bg-primary'
          : cn(
              'bg-background',
              recommended ? 'border-primary' : 'border-border',
              dimmed ? 'opacity-50' : ''
            )
      )}
      onClick={handleCardClick}
    >
      {/* Card Header */}
      <div className="flex items-center gap-3 px-4 py-[14px]">
        {/* Radio — disabled when already selected (selection is final per option) */}
        <button
          type="button"
          onClick={handleRadioClick}
          disabled={selected}
          className={cn(
            'flex h-4 w-4 shrink-0 items-center justify-center rounded-full border-2',
            selected
              ? 'border-primary-foreground'
              : 'border-border bg-background hover:border-primary transition-colors'
          )}
          aria-label={`Select ${label}`}
        >
          {selected && <span className="h-2 w-2 rounded-full bg-primary-foreground" />}
        </button>

        {/* Text column */}
        {recommended && !selected ? (
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
          <span
            className={cn(
              'flex-1 text-sm leading-[1.4]',
              selected ? 'font-medium text-primary-foreground' : 'font-normal text-foreground'
            )}
          >
            {label}
          </span>
        )}

        {/* Chevron indicator */}
        <span
          className={cn(
            'shrink-0 p-0.5',
            selected ? 'text-primary-foreground' : 'text-muted-foreground'
          )}
        >
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
            <ThumbsUp
              className={cn(
                'mt-0.5 h-3.5 w-3.5 shrink-0',
                selected ? 'text-primary-foreground' : 'text-[var(--color-success-foreground)]'
              )}
            />
            <p
              className={cn(
                'text-[13px] font-normal leading-[1.5]',
                selected ? 'text-primary-foreground' : 'text-muted-foreground'
              )}
            >
              {pros}
            </p>
          </div>
          {/* Cons */}
          <div className="flex gap-2">
            <ThumbsDown
              className={cn(
                'mt-0.5 h-3.5 w-3.5 shrink-0',
                selected ? 'text-primary-foreground' : 'text-[var(--color-error-foreground)]'
              )}
            />
            <p
              className={cn(
                'text-[13px] font-normal leading-[1.5]',
                selected ? 'text-primary-foreground' : 'text-muted-foreground'
              )}
            >
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
