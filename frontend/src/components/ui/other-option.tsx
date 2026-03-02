import * as React from 'react'
import { Send } from 'lucide-react'
import { cn } from '@/lib/utils'

interface OtherOptionProps {
  selected: boolean
  disabled: boolean
  value: string
  onChange: (v: string) => void
  onSelect: () => void
  onSubmit: () => void
}

function OtherOption({ selected, disabled, value, onChange, onSelect, onSubmit }: OtherOptionProps) {
  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      if (value.trim()) onSubmit()
    }
  }

  if (!selected) {
    return (
      <button
        type="button"
        onClick={onSelect}
        disabled={disabled}
        className={cn(
          'flex w-full items-center gap-3 rounded-[6px] border border-border bg-background px-4 py-[14px] text-left transition-colors',
          'hover:bg-accent',
          disabled && 'pointer-events-none opacity-50'
        )}
      >
        {/* Radio indicator — unselected */}
        <span className="flex h-4 w-4 shrink-0 items-center justify-center rounded-full border-2 border-border bg-background" />
        <span className="flex-1 text-sm italic leading-[1.4] text-muted-foreground">Other</span>
      </button>
    )
  }

  return (
    <div className="w-full overflow-hidden rounded-[6px]">
      {/* Header — selected state */}
      <div className="flex items-center gap-3 rounded-t-[6px] bg-primary px-4 py-[14px]">
        {/* Radio indicator — selected */}
        <span className="flex h-4 w-4 shrink-0 items-center justify-center rounded-full border-2 border-primary-foreground">
          <span className="h-2 w-2 rounded-full bg-primary-foreground" />
        </span>
        <span className="flex-1 text-sm font-medium italic leading-[1.4] text-primary-foreground">Other</span>
      </div>

      {/* Input area */}
      <div className="flex items-center gap-3 rounded-b-[6px] border-b border-l border-r border-border bg-background px-4 py-3">
        <textarea
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Describe your approach..."
          rows={2}
          autoFocus
          className="flex-1 resize-none bg-transparent text-sm leading-[1.4] text-foreground outline-none placeholder:text-muted-foreground"
        />
        <button
          type="button"
          onClick={onSubmit}
          disabled={!value.trim()}
          className={cn(
            'flex h-8 w-8 shrink-0 items-center justify-center rounded-[6px] transition-colors',
            value.trim()
              ? 'bg-primary text-primary-foreground hover:bg-primary/90'
              : 'bg-muted text-muted-foreground'
          )}
        >
          <Send className="h-4 w-4" strokeWidth={2} />
        </button>
      </div>
    </div>
  )
}

export { OtherOption }
