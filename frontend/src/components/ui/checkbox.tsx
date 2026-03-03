import * as React from 'react'
import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'

// Halo Checkbox — Checked/Unchecked, matches Checkbox/Checked and Checkbox/Unchecked
export interface CheckboxProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string
  description?: string
}

const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, label, description, id, ...props }, ref) => {
    const generatedId = React.useId()
    const checkboxId = id ?? generatedId
    return (
      <div className="flex items-start gap-2">
        <div className="relative mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center">
          <input
            type="checkbox"
            id={checkboxId}
            ref={ref}
            className="peer sr-only"
            {...props}
          />
          {/* Unchecked state */}
          <div
            className={cn(
              'h-4 w-4 rounded-[6px] border border-input bg-background transition-all duration-150',
              'peer-checked:bg-primary peer-checked:border-primary',
              'peer-focus-visible:ring-2 peer-focus-visible:ring-ring',
              'peer-disabled:cursor-not-allowed peer-disabled:opacity-50',
              className
            )}
          />
          {/* Check icon */}
          <Check className="pointer-events-none absolute size-3 text-primary-foreground scale-0 peer-checked:animate-checkmark-pop peer-checked:scale-100 transition-transform" />
        </div>
        {(label || description) && (
          <div className="flex flex-col gap-1">
            {label && (
              <label
                htmlFor={checkboxId}
                className="text-sm font-medium text-foreground leading-[1.5] cursor-pointer peer-disabled:cursor-not-allowed"
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
)
Checkbox.displayName = 'Checkbox'

export { Checkbox }
