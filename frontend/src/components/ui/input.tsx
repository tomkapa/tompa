import * as React from 'react'
import { cn } from '@/lib/utils'

// Raw Input — pill-shaped, matches Halo Input component
export type InputProps = React.InputHTMLAttributes<HTMLInputElement>

const Input = React.forwardRef<HTMLInputElement, InputProps>(
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
Input.displayName = 'Input'

// InputGroup — Label + Input, matches Halo "Input Group/Default" and "Input Group/Filled"
interface InputGroupProps extends InputProps {
  label?: string
  id?: string
}

const InputGroup = React.forwardRef<HTMLInputElement, InputGroupProps>(
  ({ label, id, className, ...props }, ref) => {
    const generatedId = React.useId()
    const inputId = id ?? generatedId
    return (
      <div className={cn('flex w-full flex-col gap-1.5', className)}>
        {label && (
          <label
            htmlFor={inputId}
            className="text-sm font-medium text-foreground leading-[1.43]"
          >
            {label}
          </label>
        )}
        <Input id={inputId} ref={ref} {...props} />
      </div>
    )
  }
)
InputGroup.displayName = 'InputGroup'

export { Input, InputGroup }
