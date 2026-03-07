import { Check, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'

interface MarkDoneButtonProps {
  onClick: () => void
  loading?: boolean
}

export function MarkDoneButton({ onClick, loading }: MarkDoneButtonProps) {
  return (
    <Button
      onClick={onClick}
      disabled={loading}
      className="w-full bg-[var(--color-success)] text-[var(--color-success-foreground)] py-[14px] h-auto hover:opacity-90 hover:shadow-none"
      leadingIcon={loading ? <Loader2 size={18} className="animate-spin" /> : <Check size={18} />}
    >
      Mark Done
    </Button>
  )
}
