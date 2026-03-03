import * as React from 'react'
import { ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils'

// Raw Select — pill-shaped with chevron icon, matches Halo Select component
export type SelectProps = React.SelectHTMLAttributes<HTMLSelectElement>

const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, children, ...props }, ref) => {
    return (
      <div className="relative flex w-full items-center">
        <select
          className={cn(
            'w-full appearance-none rounded-full border border-input bg-accent px-6 py-[18px] pr-12 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/20 focus-visible:border-primary/50 disabled:cursor-not-allowed disabled:opacity-50 transition-all duration-200',
            !props.value && !props.defaultValue && 'text-muted-foreground',
            className
          )}
          ref={ref}
          {...props}
        >
          {children}
        </select>
        <ChevronDown className="pointer-events-none absolute right-5 size-4 text-muted-foreground" />
      </div>
    )
  }
)
Select.displayName = 'Select'

// SelectGroup — Label + Select, matches Halo "Select Group/Default"
interface SelectGroupProps extends SelectProps {
  label?: string
  id?: string
  placeholder?: string
  options?: { value: string; label: string }[]
}

const SelectGroup = React.forwardRef<HTMLSelectElement, SelectGroupProps>(
  ({ label, id, placeholder, options, className, children, ...props }, ref) => {
    const generatedId = React.useId()
    const selectId = id ?? generatedId
    return (
      <div className={cn('flex w-full flex-col gap-1.5', className)}>
        {label && (
          <label htmlFor={selectId} className="text-sm font-medium text-foreground leading-[1.43]">
            {label}
          </label>
        )}
        <Select id={selectId} ref={ref} {...props}>
          {placeholder && (
            <option value="" disabled>
              {placeholder}
            </option>
          )}
          {options?.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
          {children}
        </Select>
      </div>
    )
  }
)
SelectGroup.displayName = 'SelectGroup'

export { Select, SelectGroup }
