import * as React from 'react'
import { GripVertical } from 'lucide-react'
import { cn } from '@/lib/utils'
import { StatusBadge } from './status-badge'
import { AttentionDot } from './attention-dot'
import { StoryTypeTag, type StoryType } from './story-type-tag'

export type StoryStatus = 'todo' | 'in_progress' | 'done'

export interface StoryRowData {
  id: string
  title: string
  storyType: StoryType
  status: StoryStatus
  ownerName: string
  needsAttention: boolean
}

export interface StoryTableRowProps {
  story: StoryRowData
  onClick: () => void
  dragHandleProps?: React.HTMLAttributes<HTMLDivElement>
}

export function StoryTableRow({ story, onClick, dragHandleProps }: StoryTableRowProps) {
  const isDone = story.status === 'done'

  return (
    <div
      className={cn(
        'flex items-center h-[52px] border-b border-border cursor-pointer hover:bg-accent/50 transition-colors',
        isDone && 'opacity-50'
      )}
      onClick={onClick}
      role="row"
    >
      {/* Drag Handle */}
      <div
        className="flex h-full w-10 shrink-0 items-center justify-center cursor-grab active:cursor-grabbing text-muted-foreground hover:text-foreground"
        onClick={(e) => e.stopPropagation()}
        {...dragHandleProps}
      >
        <GripVertical className="h-4 w-4" />
      </div>

      {/* Name Column */}
      <div className="flex h-full flex-1 items-center gap-2 px-3 min-w-0">
        <StoryTypeTag type={story.storyType} />
        <span className="text-sm text-foreground truncate">{story.title}</span>
        {story.needsAttention && <AttentionDot />}
      </div>

      {/* Status Column */}
      <div className="flex h-full w-[140px] shrink-0 items-center px-3">
        <StatusBadge type="story" value={story.status} />
      </div>

      {/* Owner Column */}
      <div className="hidden sm:flex h-full w-[140px] shrink-0 items-center px-3">
        <span className="text-sm text-muted-foreground truncate">{story.ownerName}</span>
      </div>
    </div>
  )
}
