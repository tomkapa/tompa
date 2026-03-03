import * as React from 'react'
import { cn } from '@/lib/utils'

// Raw Textarea — rounded-[24px] (radius-m), matches Halo Textarea component
export type TextareaProps = React.TextareaHTMLAttributes<HTMLTextAreaElement>

const Textarea = React.forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className, ...props }, ref) => {
    return (
      <textarea
        className={cn(
          'flex w-full min-h-20 rounded-[24px] border border-input bg-accent px-6 py-4 text-sm text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/20 focus-visible:border-primary/50 disabled:cursor-not-allowed disabled:opacity-50 resize-none transition-all duration-200',
          className
        )}
        ref={ref}
        {...props}
      />
    )
  }
)
Textarea.displayName = 'Textarea'

// TextareaGroup — Label + Textarea, matches Halo "Textarea Group/Default"
interface TextareaGroupProps extends TextareaProps {
  label?: string
  id?: string
}

const TextareaGroup = React.forwardRef<HTMLTextAreaElement, TextareaGroupProps>(
  ({ label, id, className, ...props }, ref) => {
    const generatedId = React.useId()
    const textareaId = id ?? generatedId
    return (
      <div className={cn('flex w-full flex-col gap-1.5', className)}>
        {label && (
          <label
            htmlFor={textareaId}
            className="text-sm font-medium text-foreground leading-[1.43]"
          >
            {label}
          </label>
        )}
        <Textarea id={textareaId} ref={ref} {...props} />
      </div>
    )
  }
)
TextareaGroup.displayName = 'TextareaGroup'

export { Textarea, TextareaGroup }
