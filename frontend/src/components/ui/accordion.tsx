import * as React from 'react'
import { ChevronDown, ChevronRight, ChevronUp } from 'lucide-react'
import { cn } from '@/lib/utils'

// Halo Accordion — Open/Closed, matches Accordion/Open and Accordion/Closed
// Trigger: title + chevron icon (up when open, down when closed)
// Content: text below trigger with padding

interface AccordionContextValue {
  openItems: Set<string>
  toggle: (id: string) => void
  type: 'single' | 'multiple'
}

const AccordionContext = React.createContext<AccordionContextValue>({
  openItems: new Set(),
  toggle: () => {},
  type: 'single',
})

interface AccordionProps {
  type?: 'single' | 'multiple'
  defaultValue?: string | string[]
  value?: string | string[]
  onValueChange?: (value: string | string[]) => void
  className?: string
  children: React.ReactNode
}

function Accordion({
  type = 'single',
  defaultValue,
  value,
  onValueChange,
  className,
  children,
}: AccordionProps) {
  const initSet = new Set<string>(
    Array.isArray(defaultValue) ? defaultValue : defaultValue ? [defaultValue] : []
  )
  const [internalOpen, setInternalOpen] = React.useState<Set<string>>(initSet)
  const controlled = value !== undefined
  const openSet = controlled
    ? new Set<string>(Array.isArray(value) ? value : value ? [value] : [])
    : internalOpen

  const toggle = (id: string) => {
    let next: Set<string>
    if (type === 'single') {
      next = openSet.has(id) ? new Set() : new Set([id])
    } else {
      next = new Set(openSet)
      if (openSet.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
    }
    if (!controlled) setInternalOpen(next)
    const arr = Array.from(next)
    onValueChange?.(type === 'single' ? arr[0] ?? '' : arr)
  }

  return (
    <AccordionContext.Provider value={{ openItems: openSet, toggle, type }}>
      <div className={cn('w-full', className)}>{children}</div>
    </AccordionContext.Provider>
  )
}

interface AccordionItemProps {
  value: string
  className?: string
  children: React.ReactNode
}

function AccordionItem({ value, className, children }: AccordionItemProps) {
  return (
    <div
      className={cn('border-b border-border', className)}
      data-state={undefined}
    >
      {React.Children.map(children, (child) => {
        if (React.isValidElement(child)) {
          return React.cloneElement(child as React.ReactElement<{ itemValue?: string }>, {
            itemValue: value,
          })
        }
        return child
      })}
    </div>
  )
}

interface AccordionTriggerProps extends React.HTMLAttributes<HTMLButtonElement> {
  itemValue?: string
  /** Put chevron on the left of children; use rightSlot for a badge/counter on the far right */
  chevronLeft?: boolean
  rightSlot?: React.ReactNode
}

function AccordionTrigger({ itemValue = '', className, children, chevronLeft, rightSlot, ...props }: AccordionTriggerProps) {
  const { openItems, toggle } = React.useContext(AccordionContext)
  const isOpen = openItems.has(itemValue)

  if (chevronLeft) {
    return (
      <button
        type="button"
        className={cn(
          'flex w-full items-center justify-between text-left focus-visible:outline-none',
          className
        )}
        onClick={() => toggle(itemValue)}
        aria-expanded={isOpen}
        {...props}
      >
        <div className="flex items-center gap-2.5">
          {isOpen
            ? <ChevronDown className="h-[18px] w-[18px] shrink-0 text-muted-foreground" />
            : <ChevronRight className="h-[18px] w-[18px] shrink-0 text-muted-foreground" />
          }
          {children}
        </div>
        {rightSlot}
      </button>
    )
  }

  return (
    <button
      type="button"
      className={cn(
        'flex w-full items-center justify-between gap-4 py-4 text-left text-base font-medium text-foreground leading-[1.5] focus-visible:outline-none',
        className
      )}
      onClick={() => toggle(itemValue)}
      aria-expanded={isOpen}
      {...props}
    >
      {children}
      {isOpen ? (
        <ChevronUp className="h-4 w-4 shrink-0 text-foreground" />
      ) : (
        <ChevronDown className="h-4 w-4 shrink-0 text-foreground" />
      )}
    </button>
  )
}

interface AccordionContentProps extends React.HTMLAttributes<HTMLDivElement> {
  itemValue?: string
}

function AccordionContent({ itemValue = '', className, children, ...props }: AccordionContentProps) {
  const { openItems } = React.useContext(AccordionContext)
  const isOpen = openItems.has(itemValue)

  if (!isOpen) return null

  return (
    <div className={cn('pb-4 text-sm text-muted-foreground leading-[1.43]', className)} {...props}>
      {children}
    </div>
  )
}

export { Accordion, AccordionItem, AccordionTrigger, AccordionContent }
