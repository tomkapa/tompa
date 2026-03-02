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
import { StoryCreation } from '@/features/stories/story-creation'
import type { StoryFormData } from '@/features/stories/story-creation'
import { ProjectSelector } from '@/features/projects/project-selector'
import { CreateProjectModal } from '@/features/projects/create-project-modal'
import type { CreateProjectFormData } from '@/features/projects/create-project-modal'
import type { Story } from '@/features/stories/stories-table'
import {
  useListStories,
  useCreateStory,
  useUpdateRank,
  getListStoriesQueryKey,
} from '@/api/generated/stories/stories'
import {
  useListProjects,
  useCreateProject,
  getListProjectsQueryKey,
} from '@/api/generated/projects/projects'
import type { StoryResponse } from '@/api/generated/tompaAPI.schemas'
import { useAuth } from '@/hooks/use-auth'
import { useSSE } from '@/hooks/use-sse'
import { useSSEStore } from '@/stores/sse-store'
import { useToastStore } from '@/stores/toast-store'

// ── Helpers ───────────────────────────────────────────────────────────────────

export function slugify(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
}

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
  searchValue: string
  onSearchChange: (value: string) => void
  hasNotification: boolean
  projectSelector: React.ReactNode
}

function AppHeader({ searchValue, onSearchChange, hasNotification, projectSelector }: AppHeaderProps) {
  return (
    <header className="flex h-16 shrink-0 items-center justify-between border-b border-border bg-background px-4 md:px-6">
      {/* Left — brand + divider + project selector */}
      <div className="flex items-center gap-3 md:gap-4">
        <div className="flex items-center gap-2">
          <div
            className="flex h-8 w-8 shrink-0 items-center justify-center rounded-[10px]"
            style={{
              background: 'linear-gradient(135deg, #5749F4 0%, #8B5CF6 100%)',
            }}
            aria-hidden
          />
          <span className="hidden text-base font-semibold leading-none text-foreground md:inline">
            Tompa
          </span>
        </div>
        <div className="hidden h-6 w-px bg-border md:block" />
        {projectSelector}
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
            placeholder="Search stories, tasks, Q&A, decisions..."
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
 * U30 — Main Application Layout (v2).
 *
 * Top-level layout for a project page. Renders:
 *   - Application header (brand, project selector, global search, user menu)
 *   - Stories table as main content
 *   - Story/Task detail modal as a fixed overlay (URL-driven via TanStack Router)
 *   - Create Project modal
 */
export function AppLayout() {
  const allParams = useParams({ strict: false }) as Record<string, string | undefined>
  const projectSlug = allParams.projectSlug ?? ''
  const storyId = allParams.storyId

  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { user } = useAuth()

  const [searchValue, setSearchValue] = React.useState('')
  const [createProjectOpen, setCreateProjectOpen] = React.useState(false)
  const [storyCreationOpen, setStoryCreationOpen] = React.useState(false)

  // ── Derive project ID from slug ───────────────────────────────────────────
  const { data: projectsResp } = useListProjects(
    undefined,
    { fetch: { credentials: 'include' } },
  )
  const projects = React.useMemo(
    () => (projectsResp?.status === 200 ? projectsResp.data : []),
    [projectsResp],
  )

  const activeProject = React.useMemo(
    () => projects.find((p) => slugify(p.name) === projectSlug),
    [projects, projectSlug],
  )
  const projectId = activeProject?.id ?? ''

  // ── SSE connection ─────────────────────────────────────────────────────────
  useSSE(projectId)
  const hasNotification = useSSEStore((s) => s.hasNotification)

  // Redirect to first project if current slug is "default" and projects are loaded
  React.useEffect(() => {
    if (projectSlug === 'default' && projects.length > 0) {
      void navigate({
        to: '/projects/$projectSlug',
        params: { projectSlug: slugify(projects[0].name) },
        replace: true,
      })
    }
  }, [projectSlug, projects, navigate])

  // ── Fetch stories ──────────────────────────────────────────────────────────
  const { data: storiesResp, isLoading: storiesLoading } = useListStories(
    { project_id: projectId },
    { query: { enabled: !!projectId } },
  )
  const stories = React.useMemo(() => {
    const apiStories = storiesResp?.status === 200 ? storiesResp.data : []
    return apiStories.map(mapStory)
  }, [storiesResp])

  // ── Filter by search ───────────────────────────────────────────────────────
  const filteredStories = React.useMemo(() => {
    if (!searchValue.trim()) return stories
    const q = searchValue.toLowerCase()
    return stories.filter((s) => s.title.toLowerCase().includes(q))
  }, [stories, searchValue])

  // ── Project mutations ─────────────────────────────────────────────────────
  const createProjectMutation = useCreateProject({
    mutation: {
      onSuccess: (resp) => {
        if ((resp.status as number) === 409) {
          useToastStore.getState().addToast({ variant: 'error', title: 'A project with that name already exists' })
          return
        }
        void queryClient.invalidateQueries({
          queryKey: getListProjectsQueryKey(),
        })
        setCreateProjectOpen(false)
        if (resp.status === 201) {
          void navigate({
            to: '/projects/$projectSlug',
            params: { projectSlug: slugify(resp.data.name) },
          })
        }
      },
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to create project' })
      },
    },
  })

  // ── Story mutations ───────────────────────────────────────────────────────
  const createStoryMutation = useCreateStory({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({
          queryKey: getListStoriesQueryKey({ project_id: projectId }),
        })
      },
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to create story' })
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
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to reorder story' })
      },
    },
  })

  // ── Callbacks ──────────────────────────────────────────────────────────────
  function handleProjectSelect(selectedProjectId: string) {
    const selected = projects.find((p) => p.id === selectedProjectId)
    if (!selected) return
    void navigate({
      to: '/projects/$projectSlug',
      params: { projectSlug: slugify(selected.name) },
    })
  }

  function handleCreateProject(data: CreateProjectFormData) {
    createProjectMutation.mutate({
      data: {
        name: data.name,
        description: data.description || undefined,
        github_repo_url: data.githubRepoUrl || undefined,
      },
    })
  }

  function handleStoryClick(clickedStoryId: string) {
    void navigate({
      to: '/projects/$projectSlug/stories/$storyId',
      params: { projectSlug, storyId: clickedStoryId },
    })
  }

  function handleNewStory() {
    setStoryCreationOpen(true)
  }

  function handleCreateStory(formData: StoryFormData) {
    createStoryMutation.mutate({
      data: {
        project_id: projectId,
        title: formData.title,
        description: formData.description,
        story_type: formData.storyType,
        owner_id: formData.ownerId || user?.user_id || '',
      },
    })
    setStoryCreationOpen(false)
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
        searchValue={searchValue}
        onSearchChange={setSearchValue}
        hasNotification={hasNotification}
        projectSelector={
          <ProjectSelector
            projects={projects}
            activeProjectId={projectId}
            onSelect={handleProjectSelect}
            onCreateNew={() => setCreateProjectOpen(true)}
          />
        }
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
          isLoading={storiesLoading}
          searchQuery={searchValue}
          className="min-h-0 flex-1"
        />
      </main>

      {/* Story / Task Detail Modal overlay */}
      {isModalOpen && <StoryModal />}

      {/* Create Project Modal */}
      <CreateProjectModal
        open={createProjectOpen}
        onOpenChange={setCreateProjectOpen}
        onSubmit={handleCreateProject}
        isLoading={createProjectMutation.isPending}
      />

      {/* Create Story Modal */}
      <StoryCreation
        open={storyCreationOpen}
        onOpenChange={setStoryCreationOpen}
        owners={user ? [{ id: user.user_id, name: user.display_name }] : []}
        isGenerating={createStoryMutation.isPending}
        onRequestExpansion={handleCreateStory}
        onApprove={(data, _editedDescription) => handleCreateStory(data)}
      />
    </div>
  )
}
