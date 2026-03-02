import * as React from 'react'
import { ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'

interface BreadcrumbSegment {
  label: string
  onClick?: () => void
}

interface AppBreadcrumbProps {
  segments: BreadcrumbSegment[]
  className?: string
}

function AppBreadcrumb({ segments, className }: AppBreadcrumbProps) {
  return (
    <nav aria-label="breadcrumb" className={cn('', className)}>
      <ol className="flex items-center gap-1">
        {segments.map((segment, index) => {
          const isLast = index === segments.length - 1
          return (
            <React.Fragment key={index}>
              <li className="inline-flex min-w-0 items-center">
                {isLast ? (
                  <span
                    aria-current="page"
                    className="max-w-[160px] truncate py-1 text-[13px] font-medium text-foreground"
                  >
                    {segment.label}
                  </span>
                ) : (
                  <button
                    onClick={segment.onClick}
                    className="max-w-[160px] truncate py-1 text-[13px] font-medium text-muted-foreground transition-colors hover:text-foreground"
                  >
                    {segment.label}
                  </button>
                )}
              </li>
              {!isLast && (
                <li
                  aria-hidden
                  className="flex h-4 w-4 shrink-0 items-center justify-center text-muted-foreground"
                >
                  <ChevronRight className="h-[14px] w-[14px]" />
                </li>
              )}
            </React.Fragment>
          )
        })}
      </ol>
    </nav>
  )
}

export { AppBreadcrumb }
export type { BreadcrumbSegment, AppBreadcrumbProps }
