import { TriangleAlert } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useExitAnimation } from '@/hooks/use-exit-animation'

type ConfirmationReason = 'pending_questions' | 'unsent_draft'

const CONTENT: Record<ConfirmationReason, { title: string; description: string }> = {
  pending_questions: {
    title: 'You have pending questions',
    description:
      'There are unanswered questions from the AI that will be lost if you leave. Are you sure you want to close?',
  },
  unsent_draft: {
    title: 'You have an unsent message',
    description:
      "You have typed a message that hasn't been sent yet. This draft will be lost if you leave. Are you sure you want to close?",
  },
}

interface ConfirmationDialogProps {
  open: boolean
  onStay: () => void
  onLeave: () => void
  reason: ConfirmationReason
}

function ConfirmationDialog({ open, onStay, onLeave, reason }: ConfirmationDialogProps) {
  const { visible, dataState } = useExitAnimation(open, 150)

  if (!visible) return null

  const { title, description } = CONTENT[reason]

  return (
    <div
      className="fixed inset-0 z-[60] flex items-center justify-center"
      role="presentation"
    >
      <div
        data-state={dataState}
        className="absolute inset-0 bg-black/40 animate-in fade-in-0 data-[state=closed]:animate-out data-[state=closed]:fade-out-0"
        aria-hidden
        onClick={onStay}
      />
      <div
        role="alertdialog"
        aria-modal
        aria-labelledby="confirm-dialog-title"
        aria-describedby="confirm-dialog-desc"
        data-state={dataState}
        className="relative z-10 w-[420px] rounded-2xl border border-border bg-card shadow-[0_8px_24px_rgba(0,0,0,0.15)] animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex flex-col gap-3 px-6 pb-4 pt-6">
          <div className="flex h-10 w-10 items-center justify-center rounded-full bg-[var(--color-warning)]">
            <TriangleAlert className="h-5 w-5 text-[var(--color-warning-foreground)]" />
          </div>
          <h2
            id="confirm-dialog-title"
            className="text-lg font-semibold text-card-foreground"
          >
            {title}
          </h2>
          <p
            id="confirm-dialog-desc"
            className="text-sm leading-relaxed text-muted-foreground"
          >
            {description}
          </p>
        </div>
        <div className="flex items-center justify-end gap-3 px-6 pb-6">
          <Button variant="outline" onClick={onStay}>
            Stay
          </Button>
          <Button variant="destructive" onClick={onLeave}>
            Leave
          </Button>
        </div>
      </div>
    </div>
  )
}

export { ConfirmationDialog }
export type { ConfirmationDialogProps, ConfirmationReason }
