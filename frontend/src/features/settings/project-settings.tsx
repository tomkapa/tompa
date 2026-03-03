import * as React from 'react'
import { useNavigate } from '@tanstack/react-router'
import { useQueryClient } from '@tanstack/react-query'
import { Copy, Check, RefreshCw, Key, Terminal } from 'lucide-react'
import { TabSwitcher } from '@/components/ui/tab-switcher'
import { Button } from '@/components/ui/button'
import { InputGroup } from '@/components/ui/input'
import { TextareaGroup } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import {
  useUpdateProject,
  getListProjectsQueryKey,
} from '@/api/generated/projects/projects'
import {
  useListKeys,
  useCreateKey,
  useRevokeKey,
  getListKeysQueryKey,
} from '@/api/generated/container-keys/container-keys'
import { useToastStore } from '@/stores/toast-store'
import type { ProjectResponse } from '@/api/generated/tompaAPI.schemas'

const TABS = [
  { id: 'project', label: 'Project Profile' },
  { id: 'registry', label: 'Container Registry' },
]

function slugify(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
}

// ── Main Settings Component ──────────────────────────────────────────────────

interface ProjectSettingsProps {
  projectId: string
  activeProject: ProjectResponse | undefined
  projectSlug: string
}

export function ProjectSettings({ projectId, activeProject, projectSlug }: ProjectSettingsProps) {
  const [activeTab, setActiveTab] = React.useState('project')

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-6 overflow-auto">
      <div className="flex shrink-0 items-center justify-between">
        <h1 className="text-xl font-semibold leading-none text-foreground md:text-2xl">Settings</h1>
      </div>

      <TabSwitcher tabs={TABS} activeId={activeTab} onChange={setActiveTab} className="self-start" />

      {activeTab === 'project' ? (
        <ProjectProfileTab projectId={projectId} activeProject={activeProject} projectSlug={projectSlug} />
      ) : (
        <ContainerRegistryTab projectId={projectId} />
      )}
    </div>
  )
}

// ── Project Profile Tab ──────────────────────────────────────────────────────

function ProjectProfileTab({
  projectId,
  activeProject,
  projectSlug,
}: {
  projectId: string
  activeProject: ProjectResponse | undefined
  projectSlug: string
}) {
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const [name, setName] = React.useState(activeProject?.name ?? '')
  const [description, setDescription] = React.useState(activeProject?.description ?? '')

  React.useEffect(() => {
    if (activeProject) {
      setName(activeProject.name)
      setDescription(activeProject.description ?? '')
    }
  }, [activeProject])

  const isDirty = name !== (activeProject?.name ?? '') || description !== (activeProject?.description ?? '')
  const canSave = isDirty && name.trim().length > 0

  const updateProjectMutation = useUpdateProject({
    mutation: {
      onSuccess: (resp) => {
        if (resp.status === 200) {
          void queryClient.invalidateQueries({ queryKey: getListProjectsQueryKey() })
          useToastStore.getState().addToast({ variant: 'success', title: 'Project updated' })
          const newSlug = slugify(resp.data.name)
          if (newSlug !== projectSlug) {
            void navigate({
              to: '/projects/$projectSlug/settings',
              params: { projectSlug: newSlug },
            })
          }
        }
      },
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to update project' })
      },
    },
  })

  function handleSave() {
    if (!projectId || !canSave) return
    updateProjectMutation.mutate({
      id: projectId,
      data: {
        name: name.trim(),
        description: description.trim() || null,
      },
    })
  }

  function handleCancel() {
    setName(activeProject?.name ?? '')
    setDescription(activeProject?.description ?? '')
  }

  return (
    <div className="rounded-2xl border border-border bg-card p-6">
      <div className="flex flex-col gap-5 max-w-lg">
        <InputGroup
          label="Project name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="My Project"
        />
        <TextareaGroup
          label="Description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Brief description of your project..."
          rows={4}
        />
        <div className="flex items-center gap-3 pt-2">
          <Button
            onClick={handleSave}
            disabled={!canSave || updateProjectMutation.isPending}
          >
            {updateProjectMutation.isPending ? 'Saving...' : 'Save changes'}
          </Button>
          <Button variant="outline" onClick={handleCancel} disabled={!isDirty}>
            Cancel
          </Button>
        </div>
      </div>
    </div>
  )
}

// ── Container Registry Tab ───────────────────────────────────────────────────

function ContainerRegistryTab({ projectId }: { projectId: string }) {
  const queryClient = useQueryClient()
  const [rawKey, setRawKey] = React.useState<string | null>(null)
  const [confirmRegenerate, setConfirmRegenerate] = React.useState(false)
  const [copied, setCopied] = React.useState(false)

  const { data: keysResp, isLoading: keysLoading } = useListKeys(
    { project_id: projectId },
    { query: { enabled: !!projectId }, fetch: { credentials: 'include' } },
  )

  const keys = React.useMemo(
    () => (keysResp?.status === 200 ? keysResp.data : []),
    [keysResp],
  )

  const activeKey = keys.find((k) => !k.revoked_at)

  const createKeyMutation = useCreateKey({
    mutation: {
      onSuccess: (resp) => {
        if (resp.status === 201) {
          setRawKey(resp.data.api_key)
          void queryClient.invalidateQueries({ queryKey: getListKeysQueryKey({ project_id: projectId }) })
          useToastStore.getState().addToast({ variant: 'success', title: 'API key generated' })
        }
      },
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to generate key' })
      },
    },
    fetch: { credentials: 'include' },
  })

  const revokeKeyMutation = useRevokeKey({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: getListKeysQueryKey({ project_id: projectId }) })
      },
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to revoke key' })
      },
    },
    fetch: { credentials: 'include' },
  })

  function handleGenerate() {
    createKeyMutation.mutate({
      data: {
        project_id: projectId,
        container_mode: 'project',
        label: 'Default',
      },
    })
  }

  function handleRegenerate() {
    if (!activeKey) return
    setConfirmRegenerate(false)
    revokeKeyMutation.mutate(
      { id: activeKey.id },
      {
        onSuccess: () => {
          createKeyMutation.mutate({
            data: {
              project_id: projectId,
              container_mode: 'project',
              label: 'Default',
            },
          })
        },
      },
    )
  }

  function handleCopy() {
    if (!rawKey) return
    void navigator.clipboard.writeText(rawKey).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  const isRegenerating = revokeKeyMutation.isPending || createKeyMutation.isPending

  if (keysLoading) {
    return (
      <div className="rounded-2xl border border-border bg-card p-6">
        <p className="text-sm text-muted-foreground">Loading keys...</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="rounded-2xl border border-border bg-card p-6">
        <div className="flex flex-col gap-5 max-w-lg">
          <div className="flex items-center gap-2">
            <Key className="h-5 w-5 text-muted-foreground" />
            <h2 className="text-base font-semibold text-foreground">Agent API Key</h2>
          </div>

          <p className="text-sm text-muted-foreground">
            This key authenticates your container agent with the Tompa server. Keep it secret.
          </p>

          {!activeKey && !rawKey ? (
            <Button
              onClick={handleGenerate}
              disabled={createKeyMutation.isPending}
              className="self-start"
            >
              {createKeyMutation.isPending ? 'Generating...' : 'Generate API Key'}
            </Button>
          ) : (
            <div className="flex flex-col gap-4">
              {/* Key display */}
              <div className="flex items-center gap-3">
                <code className="flex-1 rounded-xl bg-accent px-4 py-3 font-mono text-sm text-foreground break-all">
                  {rawKey ?? `cpk_${'*'.repeat(32)}`}
                </code>
                {rawKey && (
                  <Button variant="outline" size="icon" onClick={handleCopy} aria-label="Copy key">
                    {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                  </Button>
                )}
              </div>

              {rawKey && (
                <p className="text-xs font-medium text-amber-600">
                  Copy this key now. It won't be shown again.
                </p>
              )}

              {/* Key metadata */}
              {activeKey && (
                <div className="flex flex-wrap items-center gap-3 text-sm text-muted-foreground">
                  <Badge variant="success">Active</Badge>
                  <span>Created {new Date(activeKey.created_at).toLocaleDateString()}</span>
                  {activeKey.last_connected_at && (
                    <span>Last used {new Date(activeKey.last_connected_at).toLocaleDateString()}</span>
                  )}
                </div>
              )}

              {/* Regenerate */}
              {!confirmRegenerate ? (
                <Button
                  variant="outline"
                  className="self-start"
                  onClick={() => setConfirmRegenerate(true)}
                  disabled={isRegenerating}
                  leadingIcon={<RefreshCw className="h-4 w-4" />}
                >
                  Regenerate Key
                </Button>
              ) : (
                <div className="flex flex-col gap-3 rounded-xl border border-destructive/30 bg-destructive/5 p-4">
                  <p className="text-sm font-medium text-foreground">
                    Are you sure? This will revoke the current key immediately.
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Any running agents using the old key will be disconnected.
                  </p>
                  <div className="flex items-center gap-3">
                    <Button variant="destructive" onClick={handleRegenerate} disabled={isRegenerating}>
                      {isRegenerating ? 'Regenerating...' : 'Confirm Regenerate'}
                    </Button>
                    <Button variant="outline" onClick={() => setConfirmRegenerate(false)}>
                      Cancel
                    </Button>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Usage hint */}
      <div className="rounded-2xl border border-border bg-card p-6">
        <div className="flex flex-col gap-3 max-w-lg">
          <div className="flex items-center gap-2">
            <Terminal className="h-5 w-5 text-muted-foreground" />
            <h3 className="text-sm font-semibold text-foreground">Connection</h3>
          </div>
          <p className="text-sm text-muted-foreground">
            Set the following environment variable in your container agent:
          </p>
          <code className="rounded-xl bg-accent px-4 py-3 font-mono text-sm text-foreground break-all">
            AGENT_API_KEY={rawKey ?? 'cpk_your_key_here'}
          </code>
        </div>
      </div>
    </div>
  )
}
