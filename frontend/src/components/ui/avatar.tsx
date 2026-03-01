import * as React from 'react'
import { cn } from '@/lib/utils'

// Halo Avatar — Text (initials) or Image variant
interface AvatarProps extends React.HTMLAttributes<HTMLDivElement> {
  src?: string
  alt?: string
  initials?: string
  size?: 'default' | 'lg' | 'sm'
}

const sizeClasses = {
  sm: 'h-8 w-8 text-xs',
  default: 'h-10 w-10 text-sm',
  lg: 'h-12 w-12 text-base',
}

function Avatar({ className, src, alt, initials, size = 'default', ...props }: AvatarProps) {
  return (
    <div
      className={cn(
        'relative flex shrink-0 items-center justify-center rounded-full overflow-hidden',
        sizeClasses[size],
        src ? '' : 'bg-secondary text-secondary-foreground font-medium',
        className
      )}
      {...props}
    >
      {src ? (
        <img src={src} alt={alt ?? initials ?? 'Avatar'} className="h-full w-full object-cover" />
      ) : (
        <span>{initials}</span>
      )}
    </div>
  )
}

export { Avatar }
