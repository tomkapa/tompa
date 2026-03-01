import * as React from 'react'
import { ChevronLeft, ChevronRight, Ellipsis } from 'lucide-react'
import { cn } from '@/lib/utils'
import { buttonVariants } from './button'

// Halo Pagination — Previous/Next buttons + Page number items
// Active item: bordered circle bg-background
// Default item: no border, transparent bg

function Pagination({ className, ...props }: React.HTMLAttributes<HTMLElement>) {
  return (
    <nav
      role="navigation"
      aria-label="pagination"
      className={cn('flex items-center gap-2', className)}
      {...props}
    />
  )
}

function PaginationContent({ className, ...props }: React.HTMLAttributes<HTMLUListElement>) {
  return <ul className={cn('flex items-center gap-2', className)} {...props} />
}

function PaginationItem({ className, ...props }: React.HTMLAttributes<HTMLLIElement>) {
  return <li className={cn('', className)} {...props} />
}

interface PaginationLinkProps extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  isActive?: boolean
  disabled?: boolean
}

function PaginationLink({ className, isActive, disabled, children, ...props }: PaginationLinkProps) {
  return (
    <a
      aria-current={isActive ? 'page' : undefined}
      className={cn(
        'inline-flex h-10 w-10 items-center justify-center rounded-full text-sm transition-colors',
        isActive
          ? 'border border-border bg-background text-foreground'
          : 'bg-transparent text-foreground hover:bg-accent',
        disabled && 'pointer-events-none opacity-50',
        className
      )}
      {...props}
    >
      {children}
    </a>
  )
}

function PaginationPrevious({ className, ...props }: React.AnchorHTMLAttributes<HTMLAnchorElement>) {
  return (
    <a
      aria-label="Go to previous page"
      className={cn(
        buttonVariants({ variant: 'outline', size: 'default' }),
        'gap-1',
        className
      )}
      {...props}
    >
      <ChevronLeft className="h-4 w-4" />
      Previous
    </a>
  )
}

function PaginationNext({ className, ...props }: React.AnchorHTMLAttributes<HTMLAnchorElement>) {
  return (
    <a
      aria-label="Go to next page"
      className={cn(
        buttonVariants({ variant: 'outline', size: 'default' }),
        'gap-1',
        className
      )}
      {...props}
    >
      Next
      <ChevronRight className="h-4 w-4" />
    </a>
  )
}

function PaginationEllipsis({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      aria-hidden
      className={cn('inline-flex h-10 w-10 items-center justify-center text-muted-foreground', className)}
      {...props}
    >
      <Ellipsis className="h-4 w-4" />
    </span>
  )
}

export {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationPrevious,
  PaginationNext,
  PaginationEllipsis,
}
