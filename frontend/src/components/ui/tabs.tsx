import * as React from 'react'
import { cn } from '@/lib/utils'

// Halo Tabs — pill-shaped container with Tab Item Active/Inactive
// Container: rounded-full bg-card border border-input p-2 h-14
// Active tab: rounded-full bg-secondary shadow text-secondary-foreground
// Inactive tab: rounded-full bg-white text-accent-foreground

interface TabsContextValue {
  active: string
  setActive: (value: string) => void
}

const TabsContext = React.createContext<TabsContextValue>({ active: '', setActive: () => {} })

interface TabsProps {
  defaultValue?: string
  value?: string
  onValueChange?: (value: string) => void
  className?: string
  children: React.ReactNode
}

function Tabs({ defaultValue = '', value, onValueChange, className, children }: TabsProps) {
  const [internalActive, setInternalActive] = React.useState(defaultValue)
  const controlled = value !== undefined
  const active = controlled ? value : internalActive

  const setActive = (v: string) => {
    if (!controlled) setInternalActive(v)
    onValueChange?.(v)
  }

  return (
    <TabsContext.Provider value={{ active, setActive }}>
      <div className={cn('flex flex-col gap-4', className)}>{children}</div>
    </TabsContext.Provider>
  )
}

// Tabs list (the pill container)
function TabsList({ className, children, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      role="tablist"
      className={cn(
        'inline-flex h-14 items-center rounded-full border border-input bg-card p-2',
        className
      )}
      {...props}
    >
      {children}
    </div>
  )
}

// Individual tab trigger
interface TabsTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  value: string
}

function TabsTrigger({ value, className, children, ...props }: TabsTriggerProps) {
  const { active, setActive } = React.useContext(TabsContext)
  const isActive = active === value

  return (
    <button
      role="tab"
      type="button"
      aria-selected={isActive}
      onClick={() => setActive(value)}
      className={cn(
        'inline-flex items-center justify-center rounded-full px-6 py-2.5 text-sm transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50',
        isActive
          ? 'bg-secondary text-secondary-foreground shadow-sm font-normal'
          : 'bg-white text-accent-foreground font-normal',
        className
      )}
      {...props}
    >
      {children}
    </button>
  )
}

// Tab panel content
interface TabsContentProps extends React.HTMLAttributes<HTMLDivElement> {
  value: string
}

function TabsContent({ value, className, children, ...props }: TabsContentProps) {
  const { active } = React.useContext(TabsContext)
  if (active !== value) return null
  return (
    <div role="tabpanel" className={cn('', className)} {...props}>
      {children}
    </div>
  )
}

export { Tabs, TabsList, TabsTrigger, TabsContent }
