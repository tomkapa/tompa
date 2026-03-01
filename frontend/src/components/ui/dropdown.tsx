import * as React from 'react'
import { Check, ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils'

// Halo Dropdown — matches Dropdown, List Item/Checked, List Item/Unchecked, List Divider, List Title
// Container: bg-popover, rounded-[24px] (radius-m), shadow, border, p-2
// Items: pill-shaped, gap-2, p-[10,16]

interface DropdownContextValue {
  open: boolean
  setOpen: (open: boolean) => void
}

const DropdownContext = React.createContext<DropdownContextValue>({
  open: false,
  setOpen: () => {},
})

interface DropdownMenuProps {
  children: React.ReactNode
  className?: string
}

function DropdownMenu({ children, className }: DropdownMenuProps) {
  const [open, setOpen] = React.useState(false)
  const ref = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  return (
    <DropdownContext.Provider value={{ open, setOpen }}>
      <div ref={ref} className={cn('relative inline-block', className)}>
        {children}
      </div>
    </DropdownContext.Provider>
  )
}

function DropdownMenuTrigger({
  asChild,
  children,
  className,
  ...props
}: React.HTMLAttributes<HTMLElement> & { asChild?: boolean }) {
  const { open, setOpen } = React.useContext(DropdownContext)

  if (asChild && React.isValidElement(children)) {
    return React.cloneElement(children as React.ReactElement<React.HTMLAttributes<HTMLElement>>, {
      onClick: (e: React.MouseEvent) => {
        e.stopPropagation()
        setOpen(!open)
      },
    })
  }

  return (
    <button
      type="button"
      className={cn('inline-flex items-center gap-1.5', className)}
      onClick={(e) => {
        e.stopPropagation()
        setOpen(!open)
      }}
      {...(props as React.ButtonHTMLAttributes<HTMLButtonElement>)}
    >
      {children}
      <ChevronDown className="h-4 w-4 text-muted-foreground" />
    </button>
  )
}

function DropdownMenuContent({
  className,
  align = 'start',
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & { align?: 'start' | 'end' | 'center' }) {
  const { open } = React.useContext(DropdownContext)
  if (!open) return null

  const alignClass = {
    start: 'left-0',
    end: 'right-0',
    center: 'left-1/2 -translate-x-1/2',
  }[align]

  return (
    <div
      className={cn(
        'absolute top-full z-50 mt-2 min-w-60 rounded-[24px] border border-border bg-popover p-2 shadow-lg animate-in fade-in-0 zoom-in-95',
        alignClass,
        className
      )}
      onClick={(e) => e.stopPropagation()}
      {...props}
    >
      {children}
    </div>
  )
}

interface DropdownMenuItemProps extends React.HTMLAttributes<HTMLDivElement> {
  checked?: boolean
  disabled?: boolean
  leadingIcon?: React.ReactNode
  trailingHint?: string
}

function DropdownMenuItem({
  className,
  checked,
  disabled,
  leadingIcon,
  trailingHint,
  children,
  ...props
}: DropdownMenuItemProps) {
  const { setOpen } = React.useContext(DropdownContext)
  return (
    <div
      role="menuitem"
      className={cn(
        'flex cursor-pointer items-center justify-between rounded-full gap-2 px-4 py-2.5 text-sm text-foreground transition-colors select-none',
        'hover:bg-accent hover:text-accent-foreground',
        disabled && 'pointer-events-none opacity-50',
        className
      )}
      onClick={(e) => {
        props.onClick?.(e)
        setOpen(false)
      }}
      {...props}
    >
      <span className="flex items-center gap-2">
        {leadingIcon && (
          <span className="flex h-4 w-4 items-center justify-center">{leadingIcon}</span>
        )}
        {children}
      </span>
      <span className="flex items-center gap-2">
        {trailingHint && (
          <span className="text-xs text-foreground/60">{trailingHint}</span>
        )}
        {checked !== undefined && (
          <span className="h-4 w-4">{checked && <Check className="h-3 w-3" />}</span>
        )}
      </span>
    </div>
  )
}

function DropdownMenuSeparator({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('-mx-2 my-1 h-px bg-border', className)} {...props} />
}

function DropdownMenuLabel({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        'rounded-full px-3 py-2.5 text-xs font-medium text-muted-foreground',
        className
      )}
      {...props}
    />
  )
}

export {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuLabel,
}
