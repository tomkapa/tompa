import * as React from 'react'
import { cn } from '@/lib/utils'

// Halo Radio — Selected/Default states, matches Radio/Selected and Radio/Default

interface RadioGroupContextValue {
  name: string
  value?: string
  onChange?: (value: string) => void
}

const RadioGroupContext = React.createContext<RadioGroupContextValue>({ name: '' })

interface RadioGroupProps {
  name: string
  value?: string
  defaultValue?: string
  onChange?: (value: string) => void
  className?: string
  children: React.ReactNode
}

function RadioGroup({ name, value, defaultValue, onChange, className, children }: RadioGroupProps) {
  const [internalValue, setInternalValue] = React.useState(defaultValue ?? '')
  const controlled = value !== undefined
  const current = controlled ? value : internalValue

  const handleChange = (v: string) => {
    if (!controlled) setInternalValue(v)
    onChange?.(v)
  }

  return (
    <RadioGroupContext.Provider value={{ name, value: current, onChange: handleChange }}>
      <div role="radiogroup" className={cn('flex flex-col gap-2', className)}>
        {children}
      </div>
    </RadioGroupContext.Provider>
  )
}

interface RadioItemProps {
  value: string
  label?: string
  description?: string
  disabled?: boolean
  className?: string
}

function RadioItem({ value, label, description, disabled, className }: RadioItemProps) {
  const ctx = React.useContext(RadioGroupContext)
  const checked = ctx.value === value
  const id = React.useId()

  return (
    <div className={cn('flex items-start gap-2', className)}>
      <div className="relative mt-0.5 flex h-6 w-4 shrink-0 items-center justify-center">
        <input
          type="radio"
          id={id}
          name={ctx.name}
          value={value}
          checked={checked}
          disabled={disabled}
          onChange={() => ctx.onChange?.(value)}
          className="peer sr-only"
        />
        {/* Radio circle */}
        <div
          className={cn(
            'h-4 w-4 rounded-full border transition-colors',
            checked ? 'bg-primary border-primary' : 'bg-background border-input',
            'peer-focus-visible:ring-2 peer-focus-visible:ring-ring',
            'peer-disabled:cursor-not-allowed peer-disabled:opacity-50'
          )}
        />
        {/* Inner dot */}
        {checked && (
          <div className="pointer-events-none absolute h-1 w-1 rounded-full bg-primary-foreground" />
        )}
      </div>
      {(label || description) && (
        <div className="flex flex-col gap-1">
          {label && (
            <label
              htmlFor={id}
              className="text-sm font-medium text-foreground leading-[1.5] cursor-pointer"
            >
              {label}
            </label>
          )}
          {description && (
            <p className="text-sm text-muted-foreground leading-[1.5]">{description}</p>
          )}
        </div>
      )}
    </div>
  )
}

export { RadioGroup, RadioItem }
