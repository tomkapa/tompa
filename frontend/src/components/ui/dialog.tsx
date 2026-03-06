import * as React from 'react'
import { X } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useExitAnimation } from '@/hooks/use-exit-animation'

// Halo Dialog/Modal — Left/Center/Icon variants
// Backdrop, rounded-[24px] (radius-m), bg-card, shadow

interface DialogContextValue {
  open: boolean
  onOpenChange: (open: boolean) => void
  visible: boolean
  dataState: 'open' | 'closed'
}

const DialogContext = React.createContext<DialogContextValue>({
  open: false,
  onOpenChange: () => {},
  visible: false,
  dataState: 'closed',
})

interface DialogProps {
  open?: boolean
  defaultOpen?: boolean
  onOpenChange?: (open: boolean) => void
  children: React.ReactNode
}

function Dialog({ open, defaultOpen = false, onOpenChange, children }: DialogProps) {
  const [internalOpen, setInternalOpen] = React.useState(defaultOpen)
  const controlled = open !== undefined
  const isOpen = controlled ? open : internalOpen

  const { visible, dataState } = useExitAnimation(isOpen, 150)

  const handleOpenChange = (next: boolean) => {
    if (!controlled) setInternalOpen(next)
    onOpenChange?.(next)
  }

  return (
    <DialogContext.Provider value={{ open: isOpen, onOpenChange: handleOpenChange, visible, dataState }}>
      {children}
    </DialogContext.Provider>
  )
}

function DialogTrigger({ asChild, children, ...props }: React.HTMLAttributes<HTMLElement> & { asChild?: boolean }) {
  const { onOpenChange } = React.useContext(DialogContext)
  const child = asChild && React.isValidElement(children) ? children : undefined

  if (child) {
    return React.cloneElement(child as React.ReactElement<React.HTMLAttributes<HTMLElement>>, {
      onClick: () => onOpenChange(true),
    })
  }

  return (
    <button type="button" onClick={() => onOpenChange(true)} {...props as React.ButtonHTMLAttributes<HTMLButtonElement>}>
      {children}
    </button>
  )
}

function DialogPortal({ children }: { children: React.ReactNode }) {
  const { visible } = React.useContext(DialogContext)
  if (!visible) return null
  return <>{children}</>
}

function DialogOverlay({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  const { onOpenChange, dataState } = React.useContext(DialogContext)
  return (
    <div
      data-state={dataState}
      className={cn(
        'fixed inset-0 z-50 bg-black/50 backdrop-blur-sm',
        'animate-in fade-in-0',
        'data-[state=closed]:animate-out data-[state=closed]:fade-out-0',
        className
      )}
      onClick={() => onOpenChange(false)}
      {...props}
    />
  )
}

interface DialogContentProps extends React.HTMLAttributes<HTMLDivElement> {
  position?: 'center' | 'left'
  onClose?: () => void
}

function DialogContent({ className, position = 'center', children, onClose, ...props }: DialogContentProps) {
  const { onOpenChange, dataState } = React.useContext(DialogContext)

  const handleClose = () => {
    onOpenChange(false)
    onClose?.()
  }

  const positionClass =
    position === 'left'
      ? 'fixed left-0 top-0 z-50 h-full w-[480px] rounded-r-[24px] rounded-l-none'
      : 'fixed left-1/2 top-1/2 z-50 w-full max-w-lg -translate-x-1/2 -translate-y-1/2 rounded-[24px]'

  return (
    <DialogPortal>
      <>
        <DialogOverlay />
        <div
          role="dialog"
          aria-modal
          data-state={dataState}
          className={cn(
            'bg-card border border-border shadow-xl p-10',
            'animate-in fade-in-0 zoom-in-95',
            'data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95',
            positionClass,
            className
          )}
          onClick={(e) => e.stopPropagation()}
          {...props}
        >
          {children}
          <button
            type="button"
            className="absolute right-6 top-6 rounded-full p-1 text-muted-foreground hover:text-foreground transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onClick={handleClose}
            aria-label="Close"
          >
            <X className="h-5 w-5" />
          </button>
        </div>
      </>
    </DialogPortal>
  )
}

function DialogHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('flex flex-col gap-2 mb-6', className)} {...props} />
}

function DialogTitle({ className, ...props }: React.HTMLAttributes<HTMLHeadingElement>) {
  return (
    <h2
      className={cn('text-xl font-semibold text-card-foreground leading-[1.5]', className)}
      {...props}
    />
  )
}

function DialogDescription({ className, ...props }: React.HTMLAttributes<HTMLParagraphElement>) {
  return (
    <p className={cn('text-base text-muted-foreground leading-[1.5]', className)} {...props} />
  )
}

function DialogFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div className={cn('flex items-center justify-end gap-3 mt-6', className)} {...props} />
  )
}

function DialogClose({ asChild, children, ...props }: React.HTMLAttributes<HTMLElement> & { asChild?: boolean }) {
  const { onOpenChange } = React.useContext(DialogContext)
  const child = asChild && React.isValidElement(children) ? children : undefined

  if (child) {
    return React.cloneElement(child as React.ReactElement<React.HTMLAttributes<HTMLElement>>, {
      onClick: () => onOpenChange(false),
    })
  }

  return (
    <button type="button" onClick={() => onOpenChange(false)} {...props as React.ButtonHTMLAttributes<HTMLButtonElement>}>
      {children}
    </button>
  )
}

export {
  Dialog,
  DialogTrigger,
  DialogPortal,
  DialogOverlay,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  DialogClose,
}
