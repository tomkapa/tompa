import * as React from 'react'
import { cn } from '@/lib/utils'

// Halo Switch — Checked/Unchecked, matches Switch/Checked and Switch/Unchecked
export interface SwitchProps {
  checked?: boolean
  defaultChecked?: boolean
  onCheckedChange?: (checked: boolean) => void
  disabled?: boolean
  label?: string
  id?: string
  className?: string
}

function Switch({
  checked,
  defaultChecked,
  onCheckedChange,
  disabled,
  label,
  id,
  className,
}: SwitchProps) {
  const [internalChecked, setInternalChecked] = React.useState(defaultChecked ?? false)
  const controlled = checked !== undefined
  const isChecked = controlled ? checked : internalChecked
  const generatedId = React.useId()
  const switchId = id ?? generatedId

  const handleClick = () => {
    if (disabled) return
    const next = !isChecked
    if (!controlled) setInternalChecked(next)
    onCheckedChange?.(next)
  }

  return (
    <div className={cn('flex items-center gap-3', className)}>
      <button
        type="button"
        role="switch"
        id={switchId}
        aria-checked={isChecked}
        disabled={disabled}
        onClick={handleClick}
        className={cn(
          'relative inline-flex h-6 w-10 shrink-0 cursor-pointer items-center rounded-full p-1 transition-colors duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50',
          isChecked ? 'bg-primary' : 'bg-input'
        )}
      >
        <span
          className={cn(
            'block h-4 w-4 rounded-full bg-primary-foreground shadow-sm transition-transform duration-200 ease-[cubic-bezier(0.34,1.56,0.64,1)]',
            isChecked ? 'translate-x-4' : 'translate-x-0'
          )}
        />
      </button>
      {label && (
        <label
          htmlFor={switchId}
          className="text-sm text-foreground leading-[1.5] cursor-pointer select-none"
        >
          {label}
        </label>
      )}
    </div>
  )
}

export { Switch }
