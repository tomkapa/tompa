import * as React from 'react'
import { Info, ShieldCheck, OctagonAlert, SquareX } from 'lucide-react'
import { cn } from '@/lib/utils'

// Halo Alert — Info/Success/Warning/Error variants
// Each has: colored bg, left border, icon + title + description

type AlertVariant = 'info' | 'success' | 'warning' | 'error'

const alertConfig: Record<
  AlertVariant,
  { bgColor: string; textColor: string; borderColor: string; Icon: React.ElementType }
> = {
  info: {
    bgColor: 'bg-[var(--color-info)]',
    textColor: 'text-[var(--color-info-foreground)]',
    borderColor: 'border-l-[var(--color-info-foreground)]',
    Icon: Info,
  },
  success: {
    bgColor: 'bg-[var(--color-success)]',
    textColor: 'text-[var(--color-success-foreground)]',
    borderColor: 'border-l-[var(--color-success-foreground)]',
    Icon: ShieldCheck,
  },
  warning: {
    bgColor: 'bg-[var(--color-warning)]',
    textColor: 'text-[var(--color-warning-foreground)]',
    borderColor: 'border-l-[var(--color-warning-foreground)]',
    Icon: OctagonAlert,
  },
  error: {
    bgColor: 'bg-[var(--color-error)]',
    textColor: 'text-[var(--color-error-foreground)]',
    borderColor: 'border-l-[var(--color-error-foreground)]',
    Icon: SquareX,
  },
}

interface AlertProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: AlertVariant
  title?: string
  description?: string
}

function Alert({ variant = 'info', title, description, className, children, ...props }: AlertProps) {
  const config = alertConfig[variant]
  const { Icon } = config

  return (
    <div
      role="alert"
      className={cn(
        'flex gap-3 rounded-[24px] border-l-2 px-6 py-4',
        config.bgColor,
        config.textColor,
        config.borderColor,
        className
      )}
      {...props}
    >
      <Icon className="mt-0.5 h-6 w-6 shrink-0" />
      <div className="flex flex-col gap-1">
        {title && <p className="text-base font-medium leading-[1.5]">{title}</p>}
        {description && <p className="text-base leading-[1.5]">{description}</p>}
        {children}
      </div>
    </div>
  )
}

export { Alert }
