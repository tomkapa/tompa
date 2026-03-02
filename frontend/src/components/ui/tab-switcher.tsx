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
  return (
    <div
      className={cn(
        'inline-flex items-center gap-1 rounded-full border border-border bg-muted p-1',
        className
      )}
    >
      {tabs.map((tab) => {
        const isActive = tab.id === activeId
        return (
          <button
            key={tab.id}
            onClick={() => onChange(tab.id)}
            className={cn(
              'rounded-full px-4 py-2 text-[13px] font-medium leading-[1.4] transition-all',
              isActive
                ? 'bg-card text-foreground shadow-[0_1px_2px_rgba(0,0,0,0.1)]'
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
