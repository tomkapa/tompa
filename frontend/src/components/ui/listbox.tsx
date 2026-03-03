import * as React from 'react'
import { Check, ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useExitAnimation } from '@/hooks/use-exit-animation'

// Custom Listbox — pill-shaped trigger with popover dropdown
// Matches the design system's Input/Textarea styling for form consistency

interface ListboxOption {
  value: string
  label: string
}

interface ListboxProps {
  value: string
  onChange: (value: string) => void
  options: ListboxOption[]
  placeholder?: string
  disabled?: boolean
  className?: string
}

function Listbox({ value, onChange, options, placeholder = 'Select…', disabled, className }: ListboxProps) {
  const [open, setOpen] = React.useState(false)
  const containerRef = React.useRef<HTMLDivElement>(null)
  const { visible, dataState } = useExitAnimation(open, 150)

  const selectedLabel = options.find((o) => o.value === value)?.label

  // Close on outside click
  React.useEffect(() => {
    if (!open) return
    function handleClick(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClick)
    return () => document.removeEventListener('mousedown', handleClick)
  }, [open])

  // Close on Escape
  React.useEffect(() => {
    if (!open) return
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'Escape') setOpen(false)
    }
    document.addEventListener('keydown', handleKey)
    return () => document.removeEventListener('keydown', handleKey)
  }, [open])

  function handleSelect(optionValue: string) {
    onChange(optionValue)
    setOpen(false)
  }

  return (
    <div className={cn('relative w-full', className)} ref={containerRef}>
      {/* Trigger — styled to match Input */}
      <button
        type="button"
        disabled={disabled}
        onClick={() => setOpen((v) => !v)}
        className={cn(
          'flex w-full items-center justify-between rounded-full border border-input bg-accent px-6 py-[18px] text-sm transition-all duration-200',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/20 focus-visible:border-primary/50',
          'disabled:cursor-not-allowed disabled:opacity-50',
          selectedLabel ? 'text-foreground' : 'text-muted-foreground',
        )}
      >
        <span className="truncate">{selectedLabel ?? placeholder}</span>
        <ChevronDown
          className={cn(
            'ml-2 h-4 w-4 shrink-0 text-muted-foreground transition-transform duration-200',
            open && 'rotate-180',
          )}
        />
      </button>

      {/* Dropdown */}
      {visible && (
        <div
          data-state={dataState}
          className={cn(
            'absolute left-0 top-full z-50 mt-2 w-full rounded-2xl border border-border bg-popover p-1.5 shadow-lg',
            'animate-in fade-in-0 zoom-in-95 slide-in-from-top-2',
            'data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=closed]:slide-out-to-top-2',
          )}
        >
          {options.map((option) => {
            const isSelected = option.value === value
            return (
              <button
                key={option.value}
                type="button"
                onClick={() => handleSelect(option.value)}
                className={cn(
                  'flex w-full items-center justify-between rounded-full px-4 py-2.5 text-sm text-foreground transition-all duration-100 select-none',
                  'hover:bg-accent hover:text-accent-foreground',
                  'active:scale-[0.98] motion-reduce:transform-none',
                  isSelected && 'font-medium',
                )}
              >
                <span className="truncate">{option.label}</span>
                {isSelected && <Check className="ml-2 h-3.5 w-3.5 shrink-0 text-primary" />}
              </button>
            )
          })}
          {options.length === 0 && (
            <p className="px-4 py-2.5 text-sm text-muted-foreground">No options</p>
          )}
        </div>
      )}
    </div>
  )
}

// ListboxGroup — Label + Listbox, matches InputGroup/TextareaGroup pattern

interface ListboxGroupProps extends ListboxProps {
  label?: string
}

function ListboxGroup({ label, className, ...props }: ListboxGroupProps) {
  return (
    <div className={cn('flex w-full flex-col gap-1.5', className)}>
      {label && (
        <span className="text-sm font-medium text-foreground leading-[1.43]">
          {label}
        </span>
      )}
      <Listbox {...props} />
    </div>
  )
}

export { Listbox, ListboxGroup }
export type { ListboxOption, ListboxProps, ListboxGroupProps }
