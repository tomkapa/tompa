import { Plus } from 'lucide-react'
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core'
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
  useSortable,
  arrayMove,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { StoryTableRow, type StoryRowData } from '@/components/ui/story-table-row'

export type Story = StoryRowData

interface SortableRowProps {
  story: Story
  onStoryClick: () => void
}

function SortableRow({ story, onStoryClick }: SortableRowProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: story.id,
  })

  return (
    <div
      ref={setNodeRef}
      style={{
        transform: CSS.Transform.toString(transform),
        transition,
        opacity: isDragging ? 0.4 : 1,
        position: 'relative',
        zIndex: isDragging ? 10 : undefined,
      }}
    >
      <StoryTableRow
        story={story}
        onClick={onStoryClick}
        dragHandleProps={{ ...attributes, ...listeners }}
      />
    </div>
  )
}

function SkeletonRow() {
  return (
    <div className="flex items-center h-12 border-b border-border last:border-b-0">
      <div className="w-10 shrink-0 flex items-center justify-center">
        <div className="h-4 w-4 rounded bg-muted animate-pulse" />
      </div>
      <div className="flex-1 px-3">
        <div className="h-4 w-3/5 rounded bg-muted animate-pulse" />
      </div>
      <div className="w-[140px] shrink-0 px-3">
        <div className="h-5 w-16 rounded-full bg-muted animate-pulse" />
      </div>
      <div className="hidden sm:block w-[140px] shrink-0 px-3">
        <div className="h-4 w-20 rounded bg-muted animate-pulse" />
      </div>
    </div>
  )
}

export interface StoriesTableProps {
  stories: Story[]
  onStoryClick: (storyId: string) => void
  onNewStory: () => void
  onReorder: (storyId: string, beforeId?: string, afterId?: string) => void
  isLoading?: boolean
  searchQuery?: string
  className?: string
}

export function StoriesTable({
  stories,
  onStoryClick,
  onNewStory,
  onReorder,
  isLoading,
  searchQuery,
  className,
}: StoriesTableProps) {
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  )

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event
    if (!over || active.id === over.id) return

    const oldIndex = stories.findIndex((s) => s.id === active.id)
    const newIndex = stories.findIndex((s) => s.id === over.id)
    const reordered = arrayMove(stories, oldIndex, newIndex)

    const beforeId = reordered[newIndex - 1]?.id
    const afterId = reordered[newIndex + 1]?.id
    onReorder(String(active.id), beforeId, afterId)
  }

  return (
    <div
      className={cn(
        'rounded-2xl border border-border bg-background overflow-hidden',
        className
      )}
    >
      {/* Action bar */}
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div />
        <Button
          size="default"
          onClick={onNewStory}
          leadingIcon={<Plus className="h-4 w-4" />}
        >
          New
        </Button>
      </div>

      {/* Column headers */}
      <div className="flex items-center h-11 border-b border-border bg-muted">
        <div className="w-10 shrink-0" />
        <div className="flex-1 px-3 text-xs font-medium text-muted-foreground uppercase tracking-wide">
          Name
        </div>
        <div className="w-[140px] shrink-0 px-3 text-xs font-medium text-muted-foreground uppercase tracking-wide">
          Status
        </div>
        <div className="hidden sm:block w-[140px] shrink-0 px-3 text-xs font-medium text-muted-foreground uppercase tracking-wide">
          Owner
        </div>
      </div>

      {/* Loading skeleton */}
      {isLoading && stories.length === 0 && (
        <>
          <SkeletonRow />
          <SkeletonRow />
          <SkeletonRow />
        </>
      )}

      {/* Rows */}
      {!isLoading && (
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragEnd={handleDragEnd}
        >
          <SortableContext
            items={stories.map((s) => s.id)}
            strategy={verticalListSortingStrategy}
          >
            {stories.map((story) => (
              <SortableRow
                key={story.id}
                story={story}
                onStoryClick={() => onStoryClick(story.id)}
              />
            ))}
          </SortableContext>
        </DndContext>
      )}

      {/* Empty states */}
      {!isLoading && stories.length === 0 && (
        <div className="flex items-center justify-center h-24 text-sm text-muted-foreground">
          {searchQuery
            ? `No stories matching "${searchQuery}"`
            : 'No stories yet. Click New to add one.'}
        </div>
      )}
    </div>
  )
}
