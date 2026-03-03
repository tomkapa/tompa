import { useLayoutEffect, useRef, useState } from 'react'
import { cn } from '@/lib/utils'

interface Tab {
  id: string
  label: string
}

interface TabSwitcherProps {
  tabs: Tab[]
  activeId: string
  onChange: (id: string) => void
  className?: string
}

function TabSwitcher({ tabs, activeId, onChange, className }: TabSwitcherProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const tabRefs = useRef<Map<string, HTMLButtonElement>>(new Map())
  const [indicator, setIndicator] = useState({ left: 0, width: 0 })
  const [hasTransition, setHasTransition] = useState(false)

  useLayoutEffect(() => {
    const el = tabRefs.current.get(activeId)
    const container = containerRef.current
    if (!el || !container) return

    const containerRect = container.getBoundingClientRect()
    const elRect = el.getBoundingClientRect()

    setIndicator({
      left: elRect.left - containerRect.left,
      width: elRect.width,
    })

    // Enable transition after first measurement so initial render doesn't animate
    if (!hasTransition) {
      requestAnimationFrame(() => setHasTransition(true))
    }
  }, [activeId, tabs])

  return (
    <div
      ref={containerRef}
      className={cn(
        'relative inline-flex items-center gap-1 rounded-full border border-border bg-muted p-1',
        className
      )}
    >
      <div
        className={cn(
          'absolute top-1 h-[calc(100%-8px)] rounded-full bg-card shadow-[0_1px_2px_rgba(0,0,0,0.1)]',
          hasTransition && 'transition-[left,width] duration-300 ease-[cubic-bezier(0.4,0,0.2,1)]'
        )}
        style={{ left: indicator.left, width: indicator.width }}
      />
      {tabs.map((tab) => {
        const isActive = tab.id === activeId
        return (
          <button
            key={tab.id}
            ref={(el) => {
              if (el) tabRefs.current.set(tab.id, el)
              else tabRefs.current.delete(tab.id)
            }}
            onClick={() => onChange(tab.id)}
            className={cn(
              'relative z-[1] rounded-full px-4 py-2 text-[13px] font-medium leading-[1.4] transition-colors duration-200',
              isActive
                ? 'text-foreground'
                : 'text-muted-foreground hover:text-foreground'
            )}
          >
            {tab.label}
          </button>
        )
      })}
    </div>
  )
}

export { TabSwitcher }
export type { Tab, TabSwitcherProps }
