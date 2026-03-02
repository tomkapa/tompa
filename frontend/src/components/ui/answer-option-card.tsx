import { cn } from '@/lib/utils'

interface AnswerOptionCardProps {
  text: string
  selected: boolean
  disabled: boolean
  onSelect: () => void
}

function AnswerOptionCard({ text, selected, disabled, onSelect }: AnswerOptionCardProps) {
  return (
    <button
      type="button"
      onClick={onSelect}
      disabled={disabled}
      className={cn(
        'flex w-full items-center gap-3 rounded-[6px] border px-4 py-[14px] text-left transition-colors',
        selected
          ? 'border-primary bg-primary text-primary-foreground'
          : 'border-border bg-background text-foreground hover:bg-accent',
        disabled && !selected && 'pointer-events-none opacity-50',
        disabled && selected && 'pointer-events-none'
      )}
    >
      {/* Radio indicator */}
      <span
        className={cn(
          'flex h-4 w-4 shrink-0 items-center justify-center rounded-full border-2 transition-colors',
          selected
            ? 'border-primary-foreground bg-transparent'
            : 'border-border bg-background'
        )}
      >
        {selected && (
          <span className="h-2 w-2 rounded-full bg-primary-foreground" />
        )}
      </span>
      <span
        className={cn(
          'flex-1 text-sm leading-[1.4]',
          selected ? 'font-medium text-primary-foreground' : 'font-normal text-foreground'
        )}
      >
        {text}
      </span>
    </button>
  )
}

export { AnswerOptionCard }
