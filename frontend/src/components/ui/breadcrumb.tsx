import * as React from 'react'
import { ChevronRight, Ellipsis } from 'lucide-react'
import { cn } from '@/lib/utils'

// Halo Breadcrumb — Link/Current/Separator/Ellipsis items
// Text: 13px Inter 500, muted-foreground for links, foreground for current

function Breadcrumb({ className, ...props }: React.HTMLAttributes<HTMLElement>) {
  return <nav aria-label="breadcrumb" className={cn('', className)} {...props} />
}

function BreadcrumbList({ className, ...props }: React.HTMLAttributes<HTMLOListElement>) {
  return (
    <ol
      className={cn('flex flex-wrap items-center gap-0 text-[13px] font-medium', className)}
      {...props}
    />
  )
}

function BreadcrumbItem({ className, ...props }: React.HTMLAttributes<HTMLLIElement>) {
  return <li className={cn('inline-flex items-center', className)} {...props} />
}

function BreadcrumbLink({
  className,
  ...props
}: React.AnchorHTMLAttributes<HTMLAnchorElement>) {
  return (
    <a
      className={cn(
        'px-0 py-1 text-muted-foreground hover:text-foreground transition-colors',
        className
      )}
      {...props}
    />
  )
}

function BreadcrumbPage({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      aria-current="page"
      className={cn('px-0 py-1 text-foreground', className)}
      {...props}
    />
  )
}

function BreadcrumbSeparator({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span className={cn('flex h-4 w-4 items-center justify-center text-muted-foreground', className)} {...props}>
      <ChevronRight className="h-[14px] w-[14px]" />
    </span>
  )
}

function BreadcrumbEllipsis({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      className={cn('flex h-5 w-5 items-center justify-center text-muted-foreground', className)}
      {...props}
    >
      <Ellipsis className="h-4 w-4" />
    </span>
  )
}

export {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbPage,
  BreadcrumbSeparator,
  BreadcrumbEllipsis,
}
