import { RefreshCw, FileText } from 'lucide-react'
import { useQueryClient } from '@tanstack/react-query'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { useGetProfile, useUpdateProfile, useRegenerateProfile, getGetProfileQueryKey } from './use-profile'
import { TextSectionEditor, ListSectionEditor, KvSectionEditor } from './profile-section-editor'
import type { ProjectProfileContent } from './types'

interface ProfilePageProps {
  projectId: string
}

function ProfilePage({ projectId }: ProfilePageProps) {
  const { data: resp, isLoading, error } = useGetProfile(
    projectId,
    { fetch: { credentials: 'include' } },
  )

  const queryClient = useQueryClient()

  const updateMutation = useUpdateProfile({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: getGetProfileQueryKey(projectId) })
      },
      onError: (err) => {
        console.error('[ProfilePage] update failed', { projectId }, err)
      },
    },
    fetch: { credentials: 'include' },
  })

  const regenerateMutation = useRegenerateProfile({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: getGetProfileQueryKey(projectId) })
      },
      onError: (err) => {
        console.error('[ProfilePage] regenerate failed', { projectId }, err)
      },
    },
    fetch: { credentials: 'include' },
  })

  if (error) {
    console.error('[ProfilePage] load failed', { projectId }, error)
  }

  const profile = resp?.status === 200 ? resp.data : null

  function handleSectionChange(
    field: keyof ProjectProfileContent,
    value: ProjectProfileContent[keyof ProjectProfileContent],
  ) {
    if (!profile) return
    const updated = { ...profile.content, [field]: value }
    updateMutation.mutate({ projectId, data: { content: updated } })
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-20">
        <span className="text-sm text-muted-foreground">Loading profile...</span>
      </div>
    )
  }

  // Empty state — no profile yet (404 from backend)
  if (!profile) {
    return (
      <EmptyState
        onRegenerate={() => regenerateMutation.mutate({ projectId })}
        isRegenerating={regenerateMutation.isPending}
      />
    )
  }

  return (
    <div className="flex h-full flex-col gap-6 overflow-hidden">
      {/* Header */}
      <div className="flex shrink-0 items-center justify-between">
        <h1 className="text-xl font-semibold leading-none text-foreground md:text-2xl">
          Project Profile
        </h1>
        <Button
          variant="outline"
          leadingIcon={<RefreshCw className={regenerateMutation.isPending ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />}
          onClick={() => regenerateMutation.mutate({ projectId })}
          disabled={regenerateMutation.isPending}
        >
          {regenerateMutation.isPending ? 'Regenerating...' : 'Regenerate'}
        </Button>
      </div>

      {/* Metadata */}
      <div className="flex shrink-0 items-center gap-3">
        {profile.generated_at && (
          <Badge variant="default" className="text-xs">
            Last generated: {new Date(profile.generated_at).toLocaleDateString()}
          </Badge>
        )}
        <Badge variant={profile.generated_by === 'auto' ? 'info' : 'success'} className="text-xs">
          {profile.generated_by === 'auto' ? 'Auto-generated' : 'Manually edited'}
        </Badge>
        {profile.edited_at && (
          <span className="text-xs text-muted-foreground">
            Edited: {new Date(profile.edited_at).toLocaleDateString()}
          </span>
        )}
      </div>

      {/* Profile sections */}
      <div className="flex-1 overflow-y-auto">
        <div className="flex flex-col gap-5">
          <Card className="rounded-2xl">
            <CardHeader className="pb-2">
              <CardTitle>Identity</CardTitle>
            </CardHeader>
            <CardContent>
              <TextSectionEditor
                label=""
                value={profile.content.identity}
                onChange={(v) => handleSectionChange('identity', v)}
              />
            </CardContent>
          </Card>

          <Card className="rounded-2xl">
            <CardHeader className="pb-2">
              <CardTitle>Tech Stack</CardTitle>
            </CardHeader>
            <CardContent>
              <KvSectionEditor
                label=""
                entries={profile.content.tech_stack}
                onChange={(v) => handleSectionChange('tech_stack', v)}
              />
            </CardContent>
          </Card>

          <Card className="rounded-2xl">
            <CardHeader className="pb-2">
              <CardTitle>Architectural Patterns</CardTitle>
            </CardHeader>
            <CardContent>
              <ListSectionEditor
                label=""
                items={profile.content.architectural_patterns}
                onChange={(v) => handleSectionChange('architectural_patterns', v)}
              />
            </CardContent>
          </Card>

          <Card className="rounded-2xl">
            <CardHeader className="pb-2">
              <CardTitle>Conventions</CardTitle>
            </CardHeader>
            <CardContent>
              <ListSectionEditor
                label=""
                items={profile.content.conventions}
                onChange={(v) => handleSectionChange('conventions', v)}
              />
            </CardContent>
          </Card>

          <Card className="rounded-2xl">
            <CardHeader className="pb-2">
              <CardTitle>Team Preferences</CardTitle>
            </CardHeader>
            <CardContent>
              <ListSectionEditor
                label=""
                items={profile.content.team_preferences}
                onChange={(v) => handleSectionChange('team_preferences', v)}
              />
            </CardContent>
          </Card>

          <Card className="rounded-2xl">
            <CardHeader className="pb-2">
              <CardTitle>Domain Knowledge</CardTitle>
            </CardHeader>
            <CardContent>
              <ListSectionEditor
                label=""
                items={profile.content.domain_knowledge}
                onChange={(v) => handleSectionChange('domain_knowledge', v)}
              />
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

function EmptyState({ onRegenerate, isRegenerating }: { onRegenerate: () => void; isRegenerating: boolean }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 py-20 text-center">
      <div className="flex h-12 w-12 items-center justify-center rounded-full bg-accent">
        <FileText className="h-6 w-6 text-muted-foreground" />
      </div>
      <p className="text-sm font-medium text-foreground">No profile yet</p>
      <p className="max-w-sm text-sm text-muted-foreground">
        Complete a few stories and the profile will be auto-generated, or click Regenerate to
        create one now.
      </p>
      <Button
        leadingIcon={<RefreshCw className={isRegenerating ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />}
        onClick={onRegenerate}
        disabled={isRegenerating}
      >
        {isRegenerating ? 'Regenerating...' : 'Regenerate'}
      </Button>
    </div>
  )
}

export { ProfilePage }
