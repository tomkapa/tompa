import { Check, Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'

interface MarkDoneButtonProps {
  onClick: () => void
  loading?: boolean
}

export function MarkDoneButton({ onClick, loading }: MarkDoneButtonProps) {
  return (
    <button
      onClick={onClick}
      disabled={loading}
      className={cn(
        'inline-flex w-full items-center justify-center gap-2 rounded-full',
        'bg-[var(--color-success)] text-[var(--color-success-foreground)]',
        'px-6 py-[14px] text-sm font-semibold leading-[1.4]',
        'transition-all duration-150 hover:opacity-90 active:scale-[0.97] motion-reduce:transform-none',
        'disabled:pointer-events-none disabled:opacity-50'
      )}
    >
      {loading ? (
        <Loader2 size={18} className="animate-spin" />
      ) : (
        <Check size={18} />
      )}
      Mark Done
    </button>
  )
}
