import * as React from 'react'
import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const iconButtonVariants = cva(
  'inline-flex items-center justify-center rounded-full transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 active:scale-[0.95] motion-reduce:transform-none',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
        destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
        outline: 'border border-input bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground',
        ghost: 'bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground',
      },
      size: {
        default: 'h-10 w-10',
        lg: 'h-12 w-12',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  }
)

export interface IconButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof iconButtonVariants> {
  asChild?: boolean
}

const IconButton = React.forwardRef<HTMLButtonElement, IconButtonProps>(
  ({ className, variant, size, asChild = false, children, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button'
    return (
      <Comp
        className={cn(iconButtonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      >
        <span className="flex size-4 items-center justify-center">{children}</span>
      </Comp>
    )
  }
)
IconButton.displayName = 'IconButton'

// eslint-disable-next-line react-refresh/only-export-components
export { IconButton, iconButtonVariants }
