import * as React from 'react'
import { ChevronDown, Check, Search, Plus } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { ProjectResponse } from '@/api/generated/tompaAPI.schemas'

interface ProjectSelectorProps {
  projects: ProjectResponse[]
  activeProjectId: string
  onSelect: (projectId: string) => void
  onCreateNew: () => void
}

export function ProjectSelector({
  projects,
  activeProjectId,
  onSelect,
  onCreateNew,
}: ProjectSelectorProps) {
  const [open, setOpen] = React.useState(false)
  const [search, setSearch] = React.useState('')
  const containerRef = React.useRef<HTMLDivElement>(null)

  const activeProject = projects.find((p) => p.id === activeProjectId)

  const filtered = React.useMemo(() => {
    if (!search.trim()) return projects
    const q = search.toLowerCase()
    return projects.filter((p) => p.name.toLowerCase().includes(q))
  }, [projects, search])

  // Close on outside click
  React.useEffect(() => {
    if (!open) return
    function handleClick(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false)
        setSearch('')
      }
    }
    document.addEventListener('mousedown', handleClick)
    return () => document.removeEventListener('mousedown', handleClick)
  }, [open])

  // Close on Escape
  React.useEffect(() => {
    if (!open) return
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        setOpen(false)
        setSearch('')
      }
    }
    document.addEventListener('keydown', handleKey)
    return () => document.removeEventListener('keydown', handleKey)
  }, [open])

  function handleSelect(projectId: string) {
    onSelect(projectId)
    setOpen(false)
    setSearch('')
  }

  return (
    <div className="relative" ref={containerRef}>
      {/* Trigger */}
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex items-center gap-2 rounded-md bg-accent px-2.5 py-1.5 transition-colors hover:bg-accent/80"
      >
        <span className="h-2 w-2 shrink-0 rounded-full bg-[#A78BFA]" />
        <span className="text-sm font-medium text-foreground">
          {activeProject?.name ?? 'Select project'}
        </span>
        <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
      </button>

      {/* Dropdown */}
      {open && (
        <div className="absolute left-0 top-full z-50 mt-1 w-[260px] rounded-lg border border-border bg-popover shadow-lg">
          {/* Search */}
          <div className="p-2">
            <div className="relative">
              <Search className="pointer-events-none absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
              <input
                className="h-9 w-full rounded-full border border-input bg-accent pl-9 pr-3 text-sm text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                placeholder="Search projects..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                autoFocus
              />
            </div>
          </div>

          <div className="h-px bg-border" />

          {/* Project list */}
          <div className="p-1">
            {filtered.map((project) => (
              <button
                key={project.id}
                type="button"
                onClick={() => handleSelect(project.id)}
                className={cn(
                  'flex w-full items-center gap-2.5 rounded-md px-3 py-2.5 text-left transition-colors',
                  project.id === activeProjectId ? 'bg-accent' : 'hover:bg-accent/60',
                )}
              >
                <span className="h-2 w-2 shrink-0 rounded-full bg-[#A78BFA]" />
                <span
                  className={cn(
                    'flex-1 truncate text-sm text-foreground',
                    project.id === activeProjectId ? 'font-medium' : 'font-normal',
                  )}
                >
                  {project.name}
                </span>
                {project.id === activeProjectId && (
                  <Check className="h-3.5 w-3.5 shrink-0 text-primary" />
                )}
              </button>
            ))}
            {filtered.length === 0 && (
              <p className="px-3 py-2 text-sm text-muted-foreground">No projects found</p>
            )}
          </div>

          <div className="h-px bg-border" />

          {/* Create new */}
          <div className="p-1 pb-2">
            <button
              type="button"
              onClick={() => {
                setOpen(false)
                setSearch('')
                onCreateNew()
              }}
              className="flex w-full items-center gap-2 rounded-md px-3 py-2.5 text-left transition-colors hover:bg-accent/60"
            >
              <Plus className="h-3.5 w-3.5 text-muted-foreground" />
              <span className="text-sm text-muted-foreground">Create new project</span>
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
