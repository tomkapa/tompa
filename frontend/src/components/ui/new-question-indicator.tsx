import { ArrowDown } from 'lucide-react'
import { cn } from '@/lib/utils'

interface NewQuestionIndicatorProps {
  onClick: () => void
  visible: boolean
}

function NewQuestionIndicator({ onClick, visible }: NewQuestionIndicatorProps) {
  if (!visible) return null

  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'inline-flex items-center justify-center gap-1.5 rounded-full px-4 py-2.5',
        'bg-primary text-primary-foreground',
        'shadow-[0_4px_12px_rgba(87,73,244,0.4)]',
        'transition-opacity hover:opacity-90'
      )}
    >
      <span className="text-[13px] font-medium leading-[1.4]">New Question</span>
      <ArrowDown className="h-3.5 w-3.5" strokeWidth={2} />
    </button>
  )
}

export { NewQuestionIndicator }
