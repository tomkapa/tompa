import * as React from 'react'
import { cn } from '@/lib/utils'

// Halo Tooltip — popover on hover, matches Tooltip component
// bg-popover, rounded-[24px] (radius-m), shadow, border, padding [6,12]
interface TooltipProviderProps {
  children: React.ReactNode
}

function TooltipProvider({ children }: TooltipProviderProps) {
  return <>{children}</>
}

interface TooltipContextValue {
  open: boolean
  setOpen: (open: boolean) => void
}

const TooltipContext = React.createContext<TooltipContextValue>({
  open: false,
  setOpen: () => {},
})

interface TooltipProps {
  children: React.ReactNode
  defaultOpen?: boolean
}

function Tooltip({ children, defaultOpen = false }: TooltipProps) {
  const [open, setOpen] = React.useState(defaultOpen)
  return (
    <TooltipContext.Provider value={{ open, setOpen }}>
      <div className="relative inline-flex">{children}</div>
    </TooltipContext.Provider>
  )
}

function TooltipTrigger({
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  const { setOpen } = React.useContext(TooltipContext)
  return (
    <div
      className={cn('inline-flex', className)}
      onMouseEnter={() => setOpen(true)}
      onMouseLeave={() => setOpen(false)}
      onFocus={() => setOpen(true)}
      onBlur={() => setOpen(false)}
      {...props}
    >
      {children}
    </div>
  )
}

function TooltipContent({
  children,
  className,
  side = 'top',
  ...props
}: React.HTMLAttributes<HTMLDivElement> & { side?: 'top' | 'bottom' | 'left' | 'right' }) {
  const { open } = React.useContext(TooltipContext)
  if (!open) return null

  const positionClass = {
    top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
    bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
    left: 'right-full top-1/2 -translate-y-1/2 mr-2',
    right: 'left-full top-1/2 -translate-y-1/2 ml-2',
  }[side]

  return (
    <div
      role="tooltip"
      className={cn(
        'absolute z-50 whitespace-nowrap rounded-[24px] border border-border bg-popover px-3 py-1.5 text-sm font-medium text-popover-foreground shadow-md animate-in fade-in-0',
        positionClass,
        className
      )}
      {...props}
    >
      {children}
    </div>
  )
}

export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider }
