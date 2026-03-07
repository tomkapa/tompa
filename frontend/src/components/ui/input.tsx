import * as React from 'react'
import { cn } from '@/lib/utils'

// Private atom — pill-shaped field
const InputField = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ className, type, ...props }, ref) => {
    return (
      <input
        type={type}
        className={cn(
          'flex w-full rounded-full border border-input bg-accent px-6 py-[18px] text-sm text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/20 focus-visible:border-primary/50 disabled:cursor-not-allowed disabled:opacity-50 transition-all duration-200',
          className
        )}
        ref={ref}
        {...props}
      />
    )
  }
)
InputField.displayName = 'InputField'

// Input — optional label + field. Matches Halo "Input Group/Default" and "Input Group/Filled".
// When no label is provided, renders the field directly with no wrapper.
export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ label, id, className, ...props }, ref) => {
    const generatedId = React.useId()
    const inputId = id ?? generatedId
    if (!label) {
      return <InputField id={inputId} className={className} ref={ref} {...props} />
    }
    return (
      <div className="flex w-full flex-col gap-1.5">
        <label htmlFor={inputId} className="text-sm font-medium text-foreground leading-[1.43]">
          {label}
        </label>
        <InputField id={inputId} className={className} ref={ref} {...props} />
      </div>
    )
  }
)
Input.displayName = 'Input'

export { Input }
