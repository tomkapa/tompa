import * as React from 'react'
import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-full font-medium transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 active:scale-[0.97] motion-reduce:transform-none',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90 hover:shadow-md hover:shadow-primary/25',
        secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80 hover:shadow-sm',
        destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90 hover:shadow-md hover:shadow-destructive/25',
        outline: 'border border-input bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground hover:shadow-sm',
        ghost: 'bg-accent text-foreground hover:bg-accent/80',
        link: 'text-primary underline-offset-4 hover:underline active:scale-100',
      },
      size: {
        default: 'h-10 px-4 py-2.5 text-sm',
        lg: 'h-12 px-6 py-3 text-base',
        icon: 'h-10 w-10 text-sm',
        'icon-lg': 'h-12 w-12 text-base',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  }
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
  leadingIcon?: React.ReactNode
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, leadingIcon, children, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button'
    return (
      <Comp className={cn(buttonVariants({ variant, size, className }))} ref={ref} {...props}>
        {leadingIcon && (
          <span className="flex size-4 shrink-0 items-center justify-center">{leadingIcon}</span>
        )}
        {children}
      </Comp>
    )
  }
)
Button.displayName = 'Button'

// eslint-disable-next-line react-refresh/only-export-components
export { Button, buttonVariants }
