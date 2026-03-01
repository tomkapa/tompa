import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

// Halo "Label" / Badge component — Label/Success, Label/Warning, Label/Info, Label/Default
const badgeVariants = cva(
  'inline-flex items-center justify-center rounded-full px-3 py-2 text-sm font-medium leading-none',
  {
    variants: {
      variant: {
        success:
          'bg-[var(--color-success)] text-[var(--color-success-foreground)]',
        warning:
          'bg-[var(--color-warning)] text-[var(--color-warning-foreground)]',
        info: 'bg-[var(--color-info)] text-[var(--color-info-foreground)]',
        default: 'bg-secondary text-secondary-foreground',
        destructive: 'bg-destructive text-destructive-foreground',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  }
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return <span className={cn(badgeVariants({ variant }), className)} {...props} />
}

// eslint-disable-next-line react-refresh/only-export-components
export { Badge, badgeVariants }
