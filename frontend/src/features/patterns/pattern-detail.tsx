import * as React from 'react'
import { X, Pencil, Archive, ArrowRightLeft, ExternalLink } from 'lucide-react'
import { Link, useParams } from '@tanstack/react-router'
import { useQueryClient } from '@tanstack/react-query'
import { Button } from '@/components/ui/button'
import { IconButton } from '@/components/ui/icon-button'
import { Badge } from '@/components/ui/badge'
import { DomainTag } from '@/components/ui/domain-tag'
import { Input } from '@/components/ui/input'
import { TextareaGroup } from '@/components/ui/textarea'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  DialogClose,
} from '@/components/ui/dialog'
import { ConfidenceBar } from './confidence-bar'
import {
  useUpdatePattern,
  useRetirePattern,
  useSupersedePattern,
  getListPatternsQueryKey,
} from './use-patterns'
import type { DecisionPatternResponse } from './types'

interface PatternDetailProps {
  pattern: DecisionPatternResponse
  projectId: string
  onClose: () => void
}

function PatternDetail({ pattern, projectId, onClose }: PatternDetailProps) {
  const { projectSlug } = useParams({ strict: false }) as { projectSlug?: string }
  const [editing, setEditing] = React.useState(false)
  const [editPattern, setEditPattern] = React.useState(pattern.pattern)
  const [editRationale, setEditRationale] = React.useState(pattern.rationale)
  const [editTags, setEditTags] = React.useState(pattern.tags.join(', '))

  const [supersedeOpen, setSupersedeOpen] = React.useState(false)
  const [newPattern, setNewPattern] = React.useState('')
  const [newRationale, setNewRationale] = React.useState('')

  const queryClient = useQueryClient()

  const patchMutation = useUpdatePattern({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: getListPatternsQueryKey(projectId) })
        setEditing(false)
        onClose()
      },
      onError: (err) => {
        console.error('[PatternDetail] patch failed', { projectId, patternId: pattern.id }, err)
      },
    },
    fetch: { credentials: 'include' },
  })

  const retireMutation = useRetirePattern({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: getListPatternsQueryKey(projectId) })
        onClose()
      },
      onError: (err) => {
        console.error('[PatternDetail] retire failed', { projectId, patternId: pattern.id }, err)
      },
    },
    fetch: { credentials: 'include' },
  })

  const supersedeMutation = useSupersedePattern({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: getListPatternsQueryKey(projectId) })
        setSupersedeOpen(false)
        onClose()
      },
      onError: (err) => {
        console.error('[PatternDetail] supersede failed', { projectId, patternId: pattern.id }, err)
      },
    },
    fetch: { credentials: 'include' },
  })

  function handleSave() {
    patchMutation.mutate({
      projectId,
      patternId: pattern.id,
      data: {
        pattern: editPattern,
        rationale: editRationale,
        tags: editTags.split(',').map((t) => t.trim()).filter(Boolean),
      },
    })
  }

  function handleRetire() {
    retireMutation.mutate({ projectId, patternId: pattern.id })
  }

  function handleSupersede() {
    supersedeMutation.mutate({
      projectId,
      patternId: pattern.id,
      data: {
        pattern: newPattern,
        rationale: newRationale,
      },
    })
  }

  return (
    <>
      <div className="fixed inset-y-0 right-0 z-50 flex w-full max-w-[480px] flex-col border-l border-border bg-card shadow-lg animate-in slide-in-from-right">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-border px-6 py-4">
          <h2 className="text-base font-semibold text-foreground">Pattern Detail</h2>
          <IconButton variant="ghost" onClick={onClose} aria-label="Close">
            <X className="h-4 w-4" />
          </IconButton>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto px-6 py-5">
          <div className="flex flex-col gap-5">
            {/* Domain + confidence */}
            <div className="flex items-center gap-3">
              <DomainTag domain={pattern.domain} />
              <ConfidenceBar confidence={pattern.confidence} />
            </div>

            {/* Pattern */}
            {editing ? (
              <TextareaGroup
                label="Pattern"
                value={editPattern}
                onChange={(e) => setEditPattern(e.target.value)}
              />
            ) : (
              <div className="flex flex-col gap-1">
                <span className="text-xs font-medium text-muted-foreground">Pattern</span>
                <p className="text-sm leading-relaxed text-foreground">{pattern.pattern}</p>
              </div>
            )}

            {/* Rationale */}
            {editing ? (
              <TextareaGroup
                label="Rationale"
                value={editRationale}
                onChange={(e) => setEditRationale(e.target.value)}
              />
            ) : (
              <div className="flex flex-col gap-1">
                <span className="text-xs font-medium text-muted-foreground">Rationale</span>
                <p className="text-sm leading-relaxed text-muted-foreground">{pattern.rationale}</p>
              </div>
            )}

            {/* Tags */}
            {editing ? (
              <Input
                label="Tags (comma-separated)"
                value={editTags}
                onChange={(e) => setEditTags(e.target.value)}
              />
            ) : (
              <div className="flex flex-col gap-1.5">
                <span className="text-xs font-medium text-muted-foreground">Tags</span>
                <div className="flex flex-wrap gap-1.5">
                  {pattern.tags.map((tag) => (
                    <Badge key={tag} variant="default" className="text-xs">
                      {tag}
                    </Badge>
                  ))}
                  {pattern.tags.length === 0 && (
                    <span className="text-xs text-muted-foreground">No tags</span>
                  )}
                </div>
              </div>
            )}

            {/* Stats */}
            <div className="grid grid-cols-2 gap-4 rounded-xl border border-border bg-accent p-4">
              <div className="flex flex-col gap-0.5">
                <span className="text-xs text-muted-foreground">Usage count</span>
                <span className="text-lg font-semibold text-foreground">{pattern.usage_count}</span>
              </div>
              <div className="flex flex-col gap-0.5">
                <span className="text-xs text-muted-foreground">Override count</span>
                <span className="text-lg font-semibold text-foreground">{pattern.override_count}</span>
              </div>
            </div>

            {/* Provenance */}
            <div className="flex flex-col gap-1.5">
              <span className="text-xs font-medium text-muted-foreground">Provenance</span>
              {pattern.source_story_id && projectSlug ? (
                <div className="flex flex-col gap-1">
                  <Link
                    to="/projects/$projectSlug/stories/$storyId"
                    params={{ projectSlug, storyId: pattern.source_story_id }}
                    onClick={onClose}
                    className="inline-flex items-center gap-1 text-sm text-primary underline-offset-2 hover:underline"
                  >
                    <ExternalLink className="h-3 w-3 shrink-0" />
                    View source story
                  </Link>
                  {pattern.source_round_id && (
                    <Link
                      to="/projects/$projectSlug/stories/$storyId"
                      params={{ projectSlug, storyId: pattern.source_story_id }}
                      search={{ roundId: pattern.source_round_id }}
                      onClick={onClose}
                      className="inline-flex items-center gap-1 text-xs text-muted-foreground underline-offset-2 hover:underline"
                    >
                      <ExternalLink className="h-3 w-3 shrink-0" />
                      View source Q&A round
                    </Link>
                  )}
                </div>
              ) : (
                <span className="text-sm text-muted-foreground">No provenance linked</span>
              )}
            </div>

            {/* Timestamps */}
            <div className="flex flex-col gap-1 text-xs text-muted-foreground">
              <span>Created: {new Date(pattern.created_at).toLocaleString()}</span>
              <span>Updated: {new Date(pattern.updated_at).toLocaleString()}</span>
            </div>
          </div>
        </div>

        {/* Footer actions */}
        <div className="flex items-center gap-2 border-t border-border px-6 py-4">
          {editing ? (
            <>
              <Button variant="ghost" onClick={() => setEditing(false)}>Cancel</Button>
              <Button onClick={handleSave} disabled={patchMutation.isPending}>
                {patchMutation.isPending ? 'Saving...' : 'Save'}
              </Button>
            </>
          ) : (
            <>
              <Button
                variant="outline"
                leadingIcon={<Pencil className="h-3.5 w-3.5" />}
                onClick={() => setEditing(true)}
              >
                Edit
              </Button>
              <Button
                variant="outline"
                leadingIcon={<ArrowRightLeft className="h-3.5 w-3.5" />}
                onClick={() => setSupersedeOpen(true)}
              >
                Supersede
              </Button>
              <Button
                variant="destructive"
                leadingIcon={<Archive className="h-3.5 w-3.5" />}
                onClick={handleRetire}
                disabled={retireMutation.isPending}
              >
                {retireMutation.isPending ? 'Retiring...' : 'Retire'}
              </Button>
            </>
          )}
        </div>
      </div>

      {/* Supersede dialog */}
      <Dialog open={supersedeOpen} onOpenChange={setSupersedeOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Supersede Pattern</DialogTitle>
            <DialogDescription>
              Create a new pattern to replace the current one. The old pattern will be archived.
            </DialogDescription>
          </DialogHeader>
          <div className="flex flex-col gap-4 py-4">
            <TextareaGroup
              label="New pattern"
              value={newPattern}
              onChange={(e) => setNewPattern(e.target.value)}
              placeholder="One-sentence reusable rule..."
            />
            <TextareaGroup
              label="New rationale"
              value={newRationale}
              onChange={(e) => setNewRationale(e.target.value)}
              placeholder="Why this replaces the old pattern..."
            />
          </div>
          <DialogFooter>
            <DialogClose asChild>
              <Button variant="ghost">Cancel</Button>
            </DialogClose>
            <Button
              onClick={handleSupersede}
              disabled={!newPattern.trim() || !newRationale.trim() || supersedeMutation.isPending}
            >
              {supersedeMutation.isPending ? 'Creating...' : 'Supersede'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}

export { PatternDetail }
