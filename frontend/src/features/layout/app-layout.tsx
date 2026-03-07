import * as React from 'react'
import { Bell, Search, Plus, Filter, Settings, CircleDot, Tag, ChevronDown, ChevronUp, X, Lightbulb, FileText } from 'lucide-react'
import { useParams, useNavigate, useRouterState } from '@tanstack/react-router'
import { useQueryClient } from '@tanstack/react-query'
import { cn } from '@/lib/utils'
import { Avatar } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { IconButton } from '@/components/ui/icon-button'
import { Checkbox } from '@/components/ui/checkbox'
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
  useStartStory,
  useUpdateRank,
  getListStoriesQueryKey,
} from '@/api/generated/stories/stories'
import {
  useListProjects,
  useCreateProject,
  getListProjectsQueryKey,
} from '@/api/generated/projects/projects'
import { useListKeys } from '@/api/generated/container-keys/container-keys'
import type { StoryResponse } from '@/api/generated/tompaAPI.schemas'
import { useAuth } from '@/hooks/use-auth'
import { useSSE } from '@/hooks/use-sse'
import { useSSEStore } from '@/stores/sse-store'
import { useToastStore } from '@/stores/toast-store'
import { ProjectSettings } from '@/features/settings/project-settings'
import { PatternsPage } from '@/features/patterns/patterns-page'
import { ProfilePage } from '@/features/profile/profile-page'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  DialogClose,
} from '@/components/ui/dialog'

// ── Helpers ───────────────────────────────────────────────────────────────────

function slugify(name: string): string {
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
    ownerName: s.owner_name || 'Unassigned',
    needsAttention: s.tasks.some((t) => t.state === 'paused'),
  }
}

// ── App Header ─────────────────────────────────────────────────────────────────

interface AppHeaderProps {
  searchValue: string
  onSearchChange: (value: string) => void
  hasNotification: boolean
  projectSelector: React.ReactNode
  onSettingsClick: () => void
  onPatternsClick: () => void
  onProfileClick: () => void
  onBrandClick: () => void
  isSettingsActive?: boolean
  isPatternsActive?: boolean
  isProfileActive?: boolean
}

function AppHeader({ searchValue, onSearchChange, hasNotification, projectSelector, onSettingsClick, onPatternsClick, onProfileClick, onBrandClick, isSettingsActive, isPatternsActive, isProfileActive }: AppHeaderProps) {
  return (
    <header className="flex h-16 shrink-0 items-center justify-between border-b border-border bg-background px-4 md:px-6">
      {/* Left — brand + divider + project selector */}
      <div className="flex items-center gap-3 md:gap-4">
        <Button
          type="button"
          variant="ghost"
          onClick={onBrandClick}
          className="gap-2 rounded-lg h-auto px-0 bg-transparent hover:bg-transparent hover:opacity-80"
        >
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
        </Button>
        <div className="hidden h-6 w-px bg-border md:block" />
        {projectSelector}
      </div>

      {/* Center — global search */}
      <div className="flex items-center">
        {/* Mobile: icon-only search button */}
        <IconButton
          type="button"
          variant="ghost"
          aria-label="Search"
          className="md:hidden h-9 w-9 text-muted-foreground"
        >
          <Search className="h-4 w-4" />
        </IconButton>
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
          <IconButton
            type="button"
            variant="ghost"
            aria-label="Notifications"
            className="text-muted-foreground"
          >
            <Bell className="h-5 w-5" />
          </IconButton>
          {hasNotification && (
            <span className="absolute right-1 top-1">
              <AttentionDot />
            </span>
          )}
        </div>
        <IconButton
          type="button"
          variant="ghost"
          aria-label="Decision Patterns"
          onClick={onPatternsClick}
          className={cn(
            isPatternsActive ? 'bg-accent text-foreground' : 'text-muted-foreground'
          )}
        >
          <Lightbulb className="h-5 w-5" />
        </IconButton>
        <IconButton
          type="button"
          variant="ghost"
          aria-label="Project Profile"
          onClick={onProfileClick}
          className={cn(
            isProfileActive ? 'bg-accent text-foreground' : 'text-muted-foreground'
          )}
        >
          <FileText className="h-5 w-5" />
        </IconButton>
        <IconButton
          type="button"
          variant="ghost"
          aria-label="Settings"
          onClick={onSettingsClick}
          className={cn(
            isSettingsActive ? 'bg-accent text-foreground' : 'text-muted-foreground'
          )}
        >
          <Settings className="h-5 w-5" />
        </IconButton>
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
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const isSettingsPage = pathname.endsWith('/settings')
  const isPatternsPage = pathname.endsWith('/patterns')
  const isProfilePage = pathname.endsWith('/profile')
  const isSubPage = isSettingsPage || isPatternsPage || isProfilePage

  const [searchValue, setSearchValue] = React.useState('')
  const [createProjectOpen, setCreateProjectOpen] = React.useState(false)
  const [storyCreationOpen, setStoryCreationOpen] = React.useState(false)
  const [openFilter, setOpenFilter] = React.useState<'status' | 'type' | null>(null)
  const [filterStatus, setFilterStatus] = React.useState<Set<string>>(new Set())
  const [filterType, setFilterType] = React.useState<Set<string>>(new Set())
  const [agentNotConfiguredOpen, setAgentNotConfiguredOpen] = React.useState(false)
  const filterBarRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    if (!openFilter) return
    const handleClickOutside = (e: MouseEvent) => {
      if (filterBarRef.current && !filterBarRef.current.contains(e.target as Node)) {
        setOpenFilter(null)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [openFilter])

  const activeFilterCount = filterStatus.size + filterType.size

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

  // ── Container keys (to detect whether agent is configured) ────────────────
  const { data: keysResp } = useListKeys(
    { project_id: projectId },
    { query: { enabled: !!projectId } },
  )
  const hasAgentConfigured = React.useMemo(() => {
    if (keysResp?.status !== 200) return false
    return keysResp.data.some((k) => !k.revoked_at)
  }, [keysResp])

  // ── SSE connection ─────────────────────────────────────────────────────────
  useSSE(
    projectId,
    user?.user_id ?? null,
    React.useCallback(
      (storyId: string) => {
        void navigate({
          to: '/projects/$projectSlug/stories/$storyId',
          params: { projectSlug, storyId },
        })
      },
      [navigate, projectSlug],
    ),
  )
  const hasNotification = useSSEStore((s) => s.hasNotification)

  // Close agent dialog when navigating to sub-pages
  React.useEffect(() => {
    if (isSubPage) {
      setAgentNotConfiguredOpen(false)
    }
  }, [isSubPage])

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

  // ── Filter by search + status + type ─────────────────────────────────────
  const filteredStories = React.useMemo(() => {
    let result = stories
    if (searchValue.trim()) {
      const q = searchValue.toLowerCase()
      result = result.filter((s) => s.title.toLowerCase().includes(q))
    }
    if (filterStatus.size > 0) {
      result = result.filter((s) => filterStatus.has(s.status))
    }
    if (filterType.size > 0) {
      result = result.filter((s) => filterType.has(s.storyType))
    }
    return result
  }, [stories, searchValue, filterStatus, filterType])

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

  const startStoryMutation = useStartStory({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({
          queryKey: getListStoriesQueryKey({ project_id: projectId }),
        })
      },
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to start story' })
      },
    },
  })

  // ── Callbacks ──────────────────────────────────────────────────────────────
  function handleSettingsClick() {
    void navigate({
      to: '/projects/$projectSlug/settings',
      params: { projectSlug },
    })
  }

  function handlePatternsClick() {
    void navigate({
      to: '/projects/$projectSlug/patterns',
      params: { projectSlug },
    })
  }

  function handleProfileClick() {
    void navigate({
      to: '/projects/$projectSlug/profile',
      params: { projectSlug },
    })
  }

  function handleBrandClick() {
    void navigate({
      to: '/projects/$projectSlug',
      params: { projectSlug },
    })
  }

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

  function handleStartStory(storyId: string) {
    if (!hasAgentConfigured) {
      setAgentNotConfiguredOpen(true)
      return
    }
    startStoryMutation.mutate({ id: storyId })
  }

  function handleReorder(reorderedStoryId: string, beforeId?: string, afterId?: string) {
    // beforeId = item above the target in the new order (target goes AFTER it)
    // afterId  = item below the target in the new order (target goes BEFORE it)
    updateRankMutation.mutate({
      id: reorderedStoryId,
      data: {
        after_id: beforeId ?? undefined,
        before_id: afterId ?? undefined,
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
        onSettingsClick={handleSettingsClick}
        onPatternsClick={handlePatternsClick}
        onProfileClick={handleProfileClick}
        onBrandClick={handleBrandClick}
        isSettingsActive={isSettingsPage}
        isPatternsActive={isPatternsPage}
        isProfileActive={isProfilePage}
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
        {isSettingsPage ? (
          <ProjectSettings projectId={projectId} activeProject={activeProject} projectSlug={projectSlug} />
        ) : isPatternsPage ? (
          <PatternsPage projectId={projectId} />
        ) : isProfilePage ? (
          <ProfilePage projectId={projectId} />
        ) : (
          <>
            {/* Page header */}
            <div className="flex shrink-0 items-center justify-between">
              <h1 className="text-xl font-semibold leading-none text-foreground md:text-2xl">Stories</h1>
              <div className="flex items-center gap-2 md:gap-3">
                <Button
                  variant="outline"
                  leadingIcon={<Filter className="h-4 w-4" />}
                  className={cn('hidden sm:flex', activeFilterCount > 0 && 'border-primary text-foreground')}
                  onClick={() => setOpenFilter(openFilter ? null : 'status')}
                >
                  Filter
                  {activeFilterCount > 0 && (
                    <span className="ml-1 inline-flex h-[18px] min-w-[18px] items-center justify-center rounded-full bg-primary px-1 text-[11px] font-semibold text-primary-foreground">
                      {activeFilterCount}
                    </span>
                  )}
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

            {/* Filter bar — visible when filters are active or a dropdown is open */}
            {(activeFilterCount > 0 || openFilter) && (
              <div className="flex shrink-0 items-center gap-3" ref={filterBarRef}>
                {/* Status filter pill */}
                <div className="relative">
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => setOpenFilter(openFilter === 'status' ? null : 'status')}
                    className={cn(
                      'px-3 py-1.5 h-auto text-[13px] bg-background',
                      filterStatus.size > 0 ? 'border-primary' : 'border-border hover:bg-accent'
                    )}
                  >
                    <CircleDot className="h-3.5 w-3.5 text-muted-foreground" />
                    <span>Status</span>
                    {filterStatus.size > 0 && (
                      <span className="flex h-[18px] min-w-[18px] items-center justify-center rounded-full bg-primary px-1 text-[11px] font-semibold text-primary-foreground">
                        {filterStatus.size}
                      </span>
                    )}
                    {openFilter === 'status' ? (
                      <ChevronUp className="h-3.5 w-3.5 text-muted-foreground" />
                    ) : (
                      <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
                    )}
                  </Button>

                  {openFilter === 'status' && (
                    <div className="absolute left-0 top-full z-50 mt-2 w-[220px] rounded-2xl border border-border bg-popover p-1.5 shadow-lg animate-in fade-in-0 zoom-in-95">
                      {(['in_progress', 'todo', 'done'] as const).map((s) => (
                        <label
                          key={s}
                          className={cn(
                            'flex cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-2 transition-colors hover:bg-accent',
                            filterStatus.has(s) && 'bg-accent'
                          )}
                        >
                          <Checkbox
                            checked={filterStatus.has(s)}
                            onChange={() => setFilterStatus((prev) => {
                              const next = new Set(prev)
                              if (next.has(s)) next.delete(s); else next.add(s)
                              return next
                            })}
                          />
                          <span className={cn(
                            'inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium',
                            s === 'in_progress' && 'bg-[var(--color-info)] text-[var(--color-info-foreground)]',
                            s === 'todo' && 'bg-secondary text-secondary-foreground',
                            s === 'done' && 'bg-[var(--color-success)] text-[var(--color-success-foreground)]',
                          )}>
                            {s === 'in_progress' ? 'In Progress' : s === 'todo' ? 'To Do' : 'Done'}
                          </span>
                        </label>
                      ))}
                    </div>
                  )}
                </div>

                {/* Type filter pill */}
                <div className="relative">
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => setOpenFilter(openFilter === 'type' ? null : 'type')}
                    className={cn(
                      'px-3 py-1.5 h-auto text-[13px] bg-background',
                      filterType.size > 0 ? 'border-primary' : 'border-border hover:bg-accent'
                    )}
                  >
                    <Tag className="h-3.5 w-3.5 text-muted-foreground" />
                    <span>Type</span>
                    {filterType.size > 0 && (
                      <span className="flex h-[18px] min-w-[18px] items-center justify-center rounded-full bg-primary px-1 text-[11px] font-semibold text-primary-foreground">
                        {filterType.size}
                      </span>
                    )}
                    {openFilter === 'type' ? (
                      <ChevronUp className="h-3.5 w-3.5 text-muted-foreground" />
                    ) : (
                      <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
                    )}
                  </Button>

                  {openFilter === 'type' && (
                    <div className="absolute left-0 top-full z-50 mt-2 w-[220px] rounded-2xl border border-border bg-popover p-1.5 shadow-lg animate-in fade-in-0 zoom-in-95">
                      {(['feature', 'bug', 'refactor'] as const).map((t) => (
                        <label
                          key={t}
                          className={cn(
                            'flex cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-2 transition-colors hover:bg-accent',
                            filterType.has(t) && 'bg-accent'
                          )}
                        >
                          <Checkbox
                            checked={filterType.has(t)}
                            onChange={() => setFilterType((prev) => {
                              const next = new Set(prev)
                              if (next.has(t)) next.delete(t); else next.add(t)
                              return next
                            })}
                          />
                          <span className="capitalize text-sm">{t}</span>
                        </label>
                      ))}
                    </div>
                  )}
                </div>

                {/* Divider + active filter chips */}
                {activeFilterCount > 0 && (
                  <>
                    <div className="h-5 w-px bg-border" />
                    {Array.from(filterStatus).map((s) => (
                      <Button
                        key={`status-${s}`}
                        type="button"
                        variant="ghost"
                        onClick={() => setFilterStatus((prev) => { const next = new Set(prev); next.delete(s); return next })}
                        className={cn(
                          'gap-1.5 px-2.5 py-1 h-auto text-xs font-medium',
                          s === 'in_progress' && 'bg-[var(--color-info)] text-[var(--color-info-foreground)] hover:bg-[var(--color-info)]/80',
                          s === 'todo' && 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
                          s === 'done' && 'bg-[var(--color-success)] text-[var(--color-success-foreground)] hover:bg-[var(--color-success)]/80',
                        )}
                      >
                        {s === 'in_progress' ? 'In Progress' : s === 'todo' ? 'To Do' : 'Done'}
                        <X className="h-3 w-3" />
                      </Button>
                    ))}
                    {Array.from(filterType).map((t) => (
                      <Button
                        key={`type-${t}`}
                        type="button"
                        variant="secondary"
                        onClick={() => setFilterType((prev) => { const next = new Set(prev); next.delete(t); return next })}
                        className="gap-1.5 px-2.5 py-1 h-auto text-xs font-medium"
                      >
                        <span className="capitalize">{t}</span>
                        <X className="h-3 w-3" />
                      </Button>
                    ))}
                    <div className="flex-1" />
                    <Button
                      type="button"
                      variant="ghost"
                      onClick={() => { setFilterStatus(new Set()); setFilterType(new Set()) }}
                      className="h-auto px-0 py-0 text-[13px] text-muted-foreground hover:text-foreground bg-transparent hover:bg-transparent"
                    >
                      Clear all
                    </Button>
                  </>
                )}
              </div>
            )}

            {/* Stories table */}
            <StoriesTable
              stories={filteredStories}
              onStoryClick={handleStoryClick}
              onStartStory={handleStartStory}
              onNewStory={handleNewStory}
              onReorder={handleReorder}
              isLoading={storiesLoading}
              searchQuery={searchValue}
              className="min-h-0 flex-1"
            />
          </>
        )}
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
        onApprove={(data) => handleCreateStory(data)}
      />

      {/* Agent not configured warning */}
      <Dialog open={agentNotConfiguredOpen} onOpenChange={setAgentNotConfiguredOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Agent not configured</DialogTitle>
            <DialogDescription>
              No container agent is set up for this project. You need to configure an agent before
              starting a story — it handles grooming, planning, and implementation automatically.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <DialogClose asChild>
              <Button variant="ghost">Cancel</Button>
            </DialogClose>
            <Button
              onClick={() => {
                setAgentNotConfiguredOpen(false)
                void navigate({
                  to: '/projects/$projectSlug/settings',
                  params: { projectSlug },
                })
              }}
            >
              Go to Settings
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
