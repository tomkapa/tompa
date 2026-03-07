import { X } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useToastStore } from '@/stores/toast-store'

const variantStyles = {
  error: 'border-destructive/50 bg-destructive/10 text-destructive',
  success: 'border-green-500/50 bg-green-500/10 text-green-700 dark:text-green-400',
  info: 'border-border bg-background text-foreground',
} as const

export function Toaster() {
  const toasts = useToastStore((s) => s.toasts)
  const removeToast = useToastStore((s) => s.removeToast)

  if (toasts.length === 0) return null

  return (
    <div className="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 w-[360px] max-w-[calc(100vw-2rem)]">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={cn(
            'flex items-start gap-3 rounded-lg border px-4 py-3 shadow-lg animate-in slide-in-from-right-full fade-in duration-200',
            variantStyles[toast.variant],
          )}
        >
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium">{toast.title}</p>
            {toast.description && (
              <p className="mt-1 text-xs opacity-80">{toast.description}</p>
            )}
            {toast.action && (
              <button
                type="button"
                onClick={() => { toast.action!.onClick(); removeToast(toast.id) }}
                className="mt-2 text-xs font-medium underline underline-offset-2 opacity-80 hover:opacity-100 transition-opacity"
              >
                {toast.action.label}
              </button>
            )}
          </div>
          <button
            type="button"
            aria-label="Dismiss"
            onClick={() => removeToast(toast.id)}
            className="shrink-0 rounded-full p-0.5 opacity-60 transition-opacity hover:opacity-100"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      ))}
    </div>
  )
}
