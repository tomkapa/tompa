import * as React from 'react'
import { Send } from 'lucide-react'
import { cn } from '@/lib/utils'

interface CourseCorrectionInputProps {
  value: string
  onChange: (v: string) => void
  onSubmit: () => void
}

function CourseCorrectionInput({ value, onChange, onSubmit }: CourseCorrectionInputProps) {
  const hasValue = value.trim().length > 0

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      if (hasValue) onSubmit()
    }
  }

  return (
    <div
      className={cn(
        'flex items-center gap-3 rounded-[6px] border border-border bg-background px-4 py-3'
      )}
    >
      <textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Course-correct the AI's approach..."
        rows={1}
        className={cn(
          'flex-1 resize-none bg-transparent text-sm leading-[1.4] outline-none',
          'placeholder:text-muted-foreground',
          hasValue ? 'text-foreground' : 'text-muted-foreground'
        )}
      />
      <button
        type="button"
        onClick={onSubmit}
        disabled={!hasValue}
        className={cn(
          'flex h-8 w-8 shrink-0 items-center justify-center rounded-[6px] transition-colors',
          hasValue
            ? 'bg-primary text-primary-foreground hover:bg-primary/90'
            : 'bg-muted text-muted-foreground'
        )}
      >
        <Send className="h-4 w-4" strokeWidth={2} />
      </button>
    </div>
  )
}

export { CourseCorrectionInput }
