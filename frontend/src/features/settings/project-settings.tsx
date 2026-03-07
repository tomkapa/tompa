import * as React from 'react'
import { useNavigate } from '@tanstack/react-router'
import { useQueryClient } from '@tanstack/react-query'
import { Copy, Check, RefreshCw, Key, Terminal, Lock, Briefcase, Code, Palette, Shield, Megaphone, ChevronDown } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import { TabSwitcher } from '@/components/ui/tab-switcher'
import { Button } from '@/components/ui/button'
import { Input, InputGroup } from '@/components/ui/input'
import { TextareaGroup } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from '@/components/ui/dropdown'
import { Card, CardHeader, CardTitle, CardDescription, CardFooter } from '@/components/ui/card'
import { Accordion, AccordionTrigger, AccordionContent } from '@/components/ui/accordion'
import {
  useUpdateProject,
  useUpdateQaConfig,
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

// ── Q&A Configuration constants ───────────────────────────────────────────────

const AVAILABLE_MODELS = [
  { id: 'haiku', label: 'Haiku' },
  { id: 'sonnet', label: 'Sonnet' },
  { id: 'opus', label: 'Opus' },
] as const

const DETAIL_LEVELS = [
  { level: 1, label: 'Essential only', description: 'Only irreversible or extremely expensive decisions' },
  { level: 2, label: 'Significant', description: 'Decisions requiring days of rework' },
  { level: 3, label: 'Standard', description: 'Decisions requiring meaningful effort to reverse' },
  { level: 4, label: 'Thorough', description: 'Decisions that could cause inefficiency or debt' },
  { level: 5, label: 'Comprehensive', description: 'All decisions where professionals might disagree' },
] as const

const DEFAULT_ROLE_CONFIG = {
  model: 'sonnet',
  detail_level: 3,
  max_questions: 3,
} as const

interface GroomingRoleDef {
  id: string
  name: string
  description: string
  required: boolean
  Icon: LucideIcon
}

const GROOMING_ROLE_DEFS: GroomingRoleDef[] = [
  {
    id: 'business_analyst',
    name: 'Business Analyst',
    description: 'Requirements, acceptance criteria, business impact',
    required: true,
    Icon: Briefcase,
  },
  {
    id: 'developer',
    name: 'Developer',
    description: 'Architecture, technical feasibility, implementation details',
    required: false,
    Icon: Code,
  },
  {
    id: 'ux_designer',
    name: 'UX Designer',
    description: 'User experience, accessibility, interaction design',
    required: false,
    Icon: Palette,
  },
  {
    id: 'security_engineer',
    name: 'Security Engineer',
    description: 'Vulnerabilities, authentication, data protection',
    required: false,
    Icon: Shield,
  },
  {
    id: 'marketing',
    name: 'Marketing Specialist',
    description: 'User-facing messaging, positioning, go-to-market',
    required: false,
    Icon: Megaphone,
  },
]

type RoleConfig = { model: string; detail_level: number; max_questions: number }
type QaConfig = {
  grooming: Record<string, RoleConfig>
  planning: RoleConfig
  implementation: RoleConfig
}

function defaultQaConfig(): QaConfig {
  return {
    grooming: {
      business_analyst: { ...DEFAULT_ROLE_CONFIG },
      developer: { ...DEFAULT_ROLE_CONFIG },
      ux_designer: { ...DEFAULT_ROLE_CONFIG },
      security_engineer: { ...DEFAULT_ROLE_CONFIG },
      marketing: { ...DEFAULT_ROLE_CONFIG },
    },
    planning: { ...DEFAULT_ROLE_CONFIG },
    implementation: { model: 'sonnet', detail_level: 2, max_questions: 2 },
  }
}

function parseQaConfig(raw: Record<string, unknown>): QaConfig {
  try {
    const grooming = (raw.grooming as Record<string, RoleConfig>) ?? {}
    const planning = (raw.planning as RoleConfig) ?? { ...DEFAULT_ROLE_CONFIG }
    const implementation = (raw.implementation as RoleConfig) ?? { model: 'sonnet', detail_level: 2, max_questions: 2 }
    return { grooming, planning, implementation }
  } catch {
    return defaultQaConfig()
  }
}

const TABS = [
  { id: 'project', label: 'Project Profile' },
  { id: 'registry', label: 'Container Registry' },
  { id: 'qa', label: 'Q&A Configuration' },
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

      {activeTab === 'project' && (
        <ProjectProfileTab projectId={projectId} activeProject={activeProject} projectSlug={projectSlug} />
      )}
      {activeTab === 'registry' && (
        <ContainerRegistryTab projectId={projectId} />
      )}
      {activeTab === 'qa' && (
        <QaConfigTab projectId={projectId} activeProject={activeProject} />
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

// ── Q&A Configuration Tab ─────────────────────────────────────────────────────

function RoleConfigFields({
  config,
  onChange,
}: {
  config: RoleConfig
  onChange: (next: RoleConfig) => void
}) {
  const detailLevel = DETAIL_LEVELS.find((d) => d.level === config.detail_level) ?? DETAIL_LEVELS[2]

  return (
    <div className="flex flex-row items-start gap-4 pt-3 pb-6">
      {/* Model */}
      <div className="flex flex-1 flex-col gap-1.5">
        <label className="text-[13px] font-medium text-muted-foreground">Model</label>
        <DropdownMenu className="w-full">
          <DropdownMenuTrigger asChild>
            <Button variant="outline" className="w-full justify-between bg-background font-normal">
              <span>{AVAILABLE_MODELS.find((m) => m.id === config.model)?.label ?? 'Sonnet'}</span>
              <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent className="w-full">
            {AVAILABLE_MODELS.map((m) => (
              <DropdownMenuItem
                key={m.id}
                checked={m.id === config.model}
                onClick={() => onChange({ ...config, model: m.id })}
              >
                {m.label}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* Detail Level */}
      <div className="flex flex-1 flex-col gap-2">
        <label className="text-[13px] font-medium text-muted-foreground">Detail Level</label>
        <div className="flex gap-1">
          {DETAIL_LEVELS.map((d) => (
            <Button
              key={d.level}
              type="button"
              variant={d.level === config.detail_level ? 'default' : 'outline'}
              className="flex-1 h-9 px-0"
              title={d.description}
              onClick={() => onChange({ ...config, detail_level: d.level })}
            >
              {d.level}
            </Button>
          ))}
        </div>
        <p className="text-[11px] font-medium text-muted-foreground">{detailLevel.label}</p>
      </div>

      {/* Max Questions */}
      <div className="flex flex-1 flex-col gap-1.5">
        <label className="text-[13px] font-medium text-muted-foreground">Max Questions</label>
        <Input
          type="number"
          min={1}
          max={5}
          value={config.max_questions}
          onChange={(e) => {
            const v = Math.min(5, Math.max(1, parseInt(e.target.value, 10) || 1))
            onChange({ ...config, max_questions: v })
          }}
          className="py-2.5 px-4 bg-background"
        />
      </div>
    </div>
  )
}

function QaConfigTab({
  projectId,
  activeProject,
}: {
  projectId: string
  activeProject: ProjectResponse | undefined
}) {
  const queryClient = useQueryClient()

  const initialConfig = React.useMemo(
    () => parseQaConfig((activeProject?.qa_config as Record<string, unknown>) ?? {}),
    [activeProject],
  )

  const [config, setConfig] = React.useState<QaConfig>(initialConfig)

  React.useEffect(() => {
    setConfig(parseQaConfig((activeProject?.qa_config as Record<string, unknown>) ?? {}))
  }, [activeProject])

  const isDirty = JSON.stringify(config) !== JSON.stringify(initialConfig)
  const enabledGroomingCount = Object.keys(config.grooming).length

  const updateQaConfigMutation = useUpdateQaConfig({
    mutation: {
      onSuccess: (resp) => {
        if (resp.status === 200) {
          void queryClient.invalidateQueries({ queryKey: getListProjectsQueryKey() })
          useToastStore.getState().addToast({ variant: 'success', title: 'Q&A configuration saved' })
        }
      },
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to save Q&A configuration' })
      },
    },
    fetch: { credentials: 'include' },
  })

  function handleSave() {
    updateQaConfigMutation.mutate({ id: projectId, data: { qa_config: config } })
  }

  function handleCancel() {
    setConfig(initialConfig)
  }

  function handleToggleGroomingRole(roleId: string, enabled: boolean) {
    const next = { ...config.grooming }
    if (enabled) {
      next[roleId] = { ...DEFAULT_ROLE_CONFIG }
    } else {
      delete next[roleId]
    }
    setConfig((c) => ({ ...c, grooming: next }))
  }

  function handleGroomingRoleConfig(roleId: string, cfg: RoleConfig) {
    setConfig((c) => ({ ...c, grooming: { ...c.grooming, [roleId]: cfg } }))
  }

  return (
    <div className="flex flex-col gap-6">
      <Card className="rounded-2xl shadow-none">
        <CardHeader className="gap-1 px-10 py-8 border-b border-border">
          <CardTitle className="text-lg font-semibold">Q&amp;A Configuration</CardTitle>
          <CardDescription className="text-sm">
            Configure how AI roles behave during Q&amp;A sessions. Changes apply to all new Q&amp;A rounds in this project.
          </CardDescription>
        </CardHeader>

        <Accordion type="multiple" defaultValue={['grooming']}>
          {/* Grooming */}
          <AccordionTrigger
            itemValue="grooming"
            chevronLeft
            className="px-10 py-5 border-b border-border"
            rightSlot={
              <Badge className="px-2.5 py-1 text-[12px] text-muted-foreground">
                {enabledGroomingCount} {enabledGroomingCount === 1 ? 'role' : 'roles'}
              </Badge>
            }
          >
            <span className="text-base font-semibold text-foreground">Grooming</span>
          </AccordionTrigger>
          <AccordionContent itemValue="grooming" className="px-6 pt-2 pb-0 text-foreground">
            {GROOMING_ROLE_DEFS.map((role) => {
              const isEnabled = role.id in config.grooming
              const cfg = config.grooming[role.id]
              const { Icon } = role
              const active = isEnabled || role.required

              return (
                <div key={role.id} className="border-b border-border last:border-b-0">
                  <div className="flex items-center justify-between py-4">
                    <div className="flex flex-col gap-0.5">
                      <div className="flex items-center gap-2.5">
                        <Icon className={`h-[18px] w-[18px] text-muted-foreground transition-opacity ${active ? '' : 'opacity-50'}`} />
                        <span className={`text-sm font-semibold transition-colors ${active ? 'text-foreground' : 'text-muted-foreground'}`}>
                          {role.name}
                        </span>
                      </div>
                      <span className="pl-[28px] text-[12px] text-muted-foreground">{role.description}</span>
                    </div>
                    {role.required ? (
                      <Badge className="flex items-center gap-1.5 px-2.5 py-1 text-[12px] text-muted-foreground">
                        <Lock className="h-3 w-3" />
                        Required
                      </Badge>
                    ) : (
                      <div className="flex items-center gap-2">
                        <span className="text-[12px] font-medium text-muted-foreground">
                          {isEnabled ? 'Enabled' : 'Disabled'}
                        </span>
                        <Switch
                          checked={isEnabled}
                          onCheckedChange={(checked) => handleToggleGroomingRole(role.id, checked)}
                        />
                      </div>
                    )}
                  </div>
                  {isEnabled && cfg && (
                    <RoleConfigFields config={cfg} onChange={(next) => handleGroomingRoleConfig(role.id, next)} />
                  )}
                </div>
              )
            })}
          </AccordionContent>

          {/* Planning */}
          <AccordionTrigger
            itemValue="planning"
            chevronLeft
            className="px-10 py-5 border-b border-border"
            rightSlot={
              <Badge className="px-2.5 py-1 text-[12px] text-muted-foreground">Development only</Badge>
            }
          >
            <span className="text-base font-semibold text-foreground">Planning</span>
          </AccordionTrigger>
          <AccordionContent itemValue="planning" className="px-10 pb-0 text-foreground">
            <RoleConfigFields
              config={config.planning}
              onChange={(planning) => setConfig((c) => ({ ...c, planning }))}
            />
          </AccordionContent>

          {/* Implementation */}
          <AccordionTrigger
            itemValue="implementation"
            chevronLeft
            className="px-10 py-5"
            rightSlot={
              <Badge className="px-2.5 py-1 text-[12px] text-muted-foreground">Development only</Badge>
            }
          >
            <span className="text-base font-semibold text-foreground">Implementation</span>
          </AccordionTrigger>
          <AccordionContent itemValue="implementation" className="px-10 pb-0 text-foreground">
            <RoleConfigFields
              config={config.implementation}
              onChange={(implementation) => setConfig((c) => ({ ...c, implementation }))}
            />
          </AccordionContent>
        </Accordion>

        <CardFooter className="justify-end gap-3 border-t border-border px-10 py-4">
          <Button variant="outline" onClick={handleCancel} disabled={!isDirty}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={!isDirty || updateQaConfigMutation.isPending}>
            {updateQaConfigMutation.isPending ? 'Saving...' : 'Save Changes'}
          </Button>
        </CardFooter>
      </Card>
    </div>
  )
}
