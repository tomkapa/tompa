import * as React from 'react'
import { Bell, Search, Plus, Filter } from 'lucide-react'
import { useParams, useNavigate } from '@tanstack/react-router'
import { useQueryClient } from '@tanstack/react-query'
import { cn } from '@/lib/utils'
import { Avatar } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { AttentionDot } from '@/components/ui/attention-dot'
import { StoriesTable } from '@/features/stories/stories-table'
import { StoryModal } from '@/features/stories/story-modal'
import type { Story } from '@/features/stories/stories-table'
import {
  useListStories,
  useCreateStory,
  useUpdateRank,
  getListStoriesQueryKey,
} from '@/api/generated/stories/stories'
import type { StoryResponse } from '@/api/generated/tompaAPI.schemas'
import { useSSE } from '@/hooks/use-sse'
import { useSSEStore } from '@/stores/sse-store'

// ── Data mapping ──────────────────────────────────────────────────────────────

const KNOWN_STORY_TYPES = new Set(['feature', 'bug', 'refactor'])

function toStoryType(raw: string): Story['storyType'] {
  return KNOWN_STORY_TYPES.has(raw) ? (raw as Story['storyType']) : 'feature'
}

function toStoryStatus(raw: string): Story['status'] {
  if (raw === 'in_progress' || raw === 'done' || raw === 'todo') {
    return raw
  }
  return 'todo'
}

function mapStory(s: StoryResponse): Story {
  return {
    id: s.id,
    title: s.title,
    storyType: toStoryType(s.story_type),
    status: toStoryStatus(s.status),
    ownerName: s.owner_id,
    needsAttention: s.tasks.some((t) => t.state === 'paused'),
  }
}

// ── App Header ─────────────────────────────────────────────────────────────────

interface AppHeaderProps {
  projectName: string
  searchValue: string
  onSearchChange: (value: string) => void
  hasNotification: boolean
}

function AppHeader({ projectName, searchValue, onSearchChange, hasNotification }: AppHeaderProps) {
  return (
    <header className="flex h-16 shrink-0 items-center justify-between border-b border-border bg-background px-4 md:px-6">
      {/* Left — project icon + name */}
      <div className="flex items-center gap-3 md:gap-4">
        <div
          className="flex h-8 w-8 shrink-0 items-center justify-center rounded-[10px]"
          style={{
            background: 'linear-gradient(135deg, #5749F4 0%, #8B5CF6 100%)',
          }}
          aria-hidden
        />
        <span className="text-[18px] font-semibold leading-none text-foreground">
          {projectName}
        </span>
      </div>

      {/* Center — global search */}
      <div className="flex items-center">
        {/* Mobile: icon-only search button */}
        <button
          type="button"
          aria-label="Search"
          className="flex md:hidden h-9 w-9 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        >
          <Search className="h-4 w-4" />
        </button>
        {/* Desktop: full search bar */}
        <div className="relative hidden md:block w-[400px]">
          <Search className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            className="h-9 pl-10 pr-12 py-2 text-sm rounded-full"
            placeholder="Search stories, tasks, Q&A, decisions…"
            value={searchValue}
            onChange={(e) => onSearchChange(e.target.value)}
            aria-label="Search"
          />
          <span className="pointer-events-none absolute right-4 top-1/2 -translate-y-1/2 rounded bg-muted px-1.5 py-0.5 text-[11px] font-medium text-muted-foreground">
            ⌘K
          </span>
        </div>
      </div>

      {/* Right — notification bell + user avatar */}
      <div className="flex items-center gap-2 md:gap-3">
        <div className="relative">
          <button
            type="button"
            aria-label="Notifications"
            className="flex h-10 w-10 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
          >
            <Bell className="h-5 w-5" />
          </button>
          {hasNotification && (
            <span className="absolute right-1 top-1">
              <AttentionDot />
            </span>
          )}
        </div>
        <Avatar initials="JD" size="default" />
      </div>
    </header>
  )
}

// ── App Layout ────────────────────────────────────────────────────────────────

/**
 * U30 — Main Application Layout.
 *
 * Top-level layout for a project page. Renders:
 *   - Application header (project name, global search, user menu)
 *   - Stories table as main content
 *   - Story/Task detail modal as a fixed overlay (URL-driven via TanStack Router)
 *
 * Three visual states:
 *   1. Default   — table only
 *   2. Modal     — table + story/task modal overlay
 *   3. Notification — table + header notification dot
 */
export function AppLayout() {
  const allParams = useParams({ strict: false }) as Record<string, string | undefined>
  const projectId = allParams.projectId ?? ''
  const storyId = allParams.storyId

  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const [searchValue, setSearchValue] = React.useState('')

  // ── SSE connection ─────────────────────────────────────────────────────────
  useSSE(projectId)
  const hasNotification = useSSEStore((s) => s.hasNotification)

  // ── Fetch stories ──────────────────────────────────────────────────────────
  const { data: storiesResp } = useListStories(
    { project_id: projectId },
    { query: { enabled: !!projectId } },
  )
  const apiStories = storiesResp?.status === 200 ? storiesResp.data : []
  const stories = React.useMemo(() => apiStories.map(mapStory), [apiStories])

  // ── Filter by search ───────────────────────────────────────────────────────
  const filteredStories = React.useMemo(() => {
    if (!searchValue.trim()) return stories
    const q = searchValue.toLowerCase()
    return stories.filter((s) => s.title.toLowerCase().includes(q))
  }, [stories, searchValue])

  // ── Mutations ──────────────────────────────────────────────────────────────
  const createStoryMutation = useCreateStory({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({
          queryKey: getListStoriesQueryKey({ project_id: projectId }),
        })
      },
    },
  })

  const updateRankMutation = useUpdateRank({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({
          queryKey: getListStoriesQueryKey({ project_id: projectId }),
        })
      },
    },
  })

  // ── Callbacks ──────────────────────────────────────────────────────────────
  function handleStoryClick(clickedStoryId: string) {
    void navigate({
      to: '/projects/$projectId/stories/$storyId',
      params: { projectId, storyId: clickedStoryId },
    })
  }

  function handleNewStory() {
    createStoryMutation.mutate({
      data: {
        project_id: projectId,
        title: 'New Story',
        description: '',
        story_type: 'feature',
        owner_id: '',
      },
    })
  }

  function handleReorder(reorderedStoryId: string, beforeId?: string, afterId?: string) {
    updateRankMutation.mutate({
      id: reorderedStoryId,
      data: {
        before_id: beforeId ?? undefined,
        after_id: afterId ?? undefined,
      },
    })
  }

  const isModalOpen = !!storyId

  return (
    <div className={cn('flex h-screen flex-col overflow-hidden bg-background')}>
      <AppHeader
        projectName={projectId || 'Tompa'}
        searchValue={searchValue}
        onSearchChange={setSearchValue}
        hasNotification={hasNotification}
      />

      {/* Main content */}
      <main className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden bg-accent p-4 md:gap-6 md:p-8">
        {/* Page header */}
        <div className="flex shrink-0 items-center justify-between">
          <h1 className="text-xl font-semibold leading-none text-foreground md:text-2xl">Stories</h1>
          <div className="flex items-center gap-2 md:gap-3">
            <Button
              variant="outline"
              leadingIcon={<Filter className="h-4 w-4" />}
              className="hidden sm:flex"
            >
              Filter
            </Button>
            <Button
              leadingIcon={<Plus className="h-4 w-4" />}
              onClick={handleNewStory}
              disabled={createStoryMutation.isPending}
            >
              New Story
            </Button>
          </div>
        </div>

        {/* Stories table */}
        <StoriesTable
          stories={filteredStories}
          onStoryClick={handleStoryClick}
          onNewStory={handleNewStory}
          onReorder={handleReorder}
          className="min-h-0 flex-1"
        />
      </main>

      {/* Story / Task Detail Modal overlay */}
      {isModalOpen && <StoryModal />}
    </div>
  )
}
