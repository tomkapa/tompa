import { useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { Building2, Users, Zap, Kanban, Sparkles, GitBranch, CirclePlus } from 'lucide-react'
import { useAuth } from '@/hooks/use-auth'
import { markOnboardingComplete } from './onboarding-storage'
import {
  Button,
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  InputGroup,
  SelectGroup,
  TextareaGroup,
  Progress,
  RadioGroup,
  RadioItem,
} from '@/components/ui'

const FEATURES_ORG = [
  { icon: Building2, text: 'Organize your teams and projects' },
  { icon: Users, text: 'Invite collaborators seamlessly' },
  { icon: Zap, text: 'AI-driven workflow automation' },
] as const

const FEATURES_PROJECT = [
  { icon: Kanban, text: 'AI-powered story decomposition' },
  { icon: Sparkles, text: 'Smart task management' },
  { icon: GitBranch, text: 'Interactive Q&A workflows' },
] as const

const TEAM_SIZE_OPTIONS = [
  { value: 'Just me', label: 'Just me' },
  { value: '2–5', label: '2–5' },
  { value: '6–15', label: '6–15' },
  { value: '16–50', label: '16–50' },
  { value: '51–200', label: '51–200' },
  { value: '200+', label: '200+' },
]

function slugify(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

async function updateCurrentOrg(name: string): Promise<void> {
  const res = await fetch('/api/v1/orgs/current', {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ name }),
  })
  if (!res.ok) throw new Error('Failed to update organization')
}

async function createProject(name: string, description: string): Promise<string> {
  const res = await fetch('/api/v1/projects', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ name, description: description.trim() || null }),
  })
  if (!res.ok) throw new Error('Failed to create project')
  const data = await res.json()
  return data.id as string
}

export function OnboardingPage() {
  const { user } = useAuth()
  const navigate = useNavigate()
  const [step, setStep] = useState<1 | 2 | 3>(1)

  // Step 2 state
  const [orgName, setOrgName] = useState('')
  const [teamSize, setTeamSize] = useState('')

  // Step 3 state
  const [projectName, setProjectName] = useState('')
  const [projectDescription, setProjectDescription] = useState('')

  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const features = step === 3 ? FEATURES_PROJECT : FEATURES_ORG

  async function handleContinue() {
    setError(null)

    if (step === 1) {
      setStep(2)
      return
    }

    if (step === 2) {
      if (!orgName.trim()) {
        setError('Organization name is required')
        return
      }
      setIsSubmitting(true)
      try {
        await updateCurrentOrg(orgName.trim())
        setStep(3)
      } catch {
        setError('Something went wrong. Please try again.')
      } finally {
        setIsSubmitting(false)
      }
      return
    }

    // Step 3 — create project
    if (!projectName.trim()) {
      setError('Project name is required')
      return
    }
    setIsSubmitting(true)
    try {
      await createProject(projectName.trim(), projectDescription)
      if (user) markOnboardingComplete(user.user_id)
      navigate({ to: '/projects/$projectSlug', params: { projectSlug: 'default' } })
    } catch {
      setError('Something went wrong. Please try again.')
    } finally {
      setIsSubmitting(false)
    }
  }

  function handleBack() {
    setError(null)
    if (step === 2) setStep(1)
    else if (step === 3) setStep(2)
  }

  const continueLabel = isSubmitting
    ? step === 3 ? 'Creating…' : 'Setting up…'
    : step === 3 ? 'Create Project' : 'Continue'

  return (
    <div className="flex h-screen bg-[#131124]">
      {/* Left Panel — Branding */}
      <div className="hidden lg:flex w-[640px] shrink-0 flex-col items-center justify-center gap-8 bg-primary p-12">
        <div className="flex flex-col items-center gap-4">
          <div className="flex items-center gap-3">
            <span className="material-symbols-rounded text-[32px] text-primary-foreground leading-none">
              hub
            </span>
            <span className="text-[28px] font-bold leading-tight text-primary-foreground">
              Tompa
            </span>
          </div>
          <p className="text-base text-primary-foreground/80 text-center">
            AI-Powered Project Management
          </p>
        </div>

        <div className="h-[300px] w-[400px] rounded-3xl bg-primary-foreground/10 flex items-center justify-center overflow-hidden">
          <span className="text-sm font-medium text-primary-foreground/40">Intelligent Workflows</span>
        </div>

        <div className="flex flex-col gap-4 w-[360px]">
          {features.map(({ icon: Icon, text }) => (
            <div key={text} className="flex items-center gap-3">
              <Icon className="h-5 w-5 shrink-0 text-primary-foreground/80" />
              <span className="text-sm text-primary-foreground/80">{text}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Right Panel */}
      <div className="flex flex-1 flex-col items-center justify-center gap-8 p-12">
        {/* Step Indicator */}
        <div className="flex flex-col items-center gap-4">
          <span className="text-[13px] font-medium text-muted-foreground tracking-[0.5px]">
            Step {step} of 3
          </span>
          <Progress
            value={step}
            max={3}
            className="w-[200px] h-1 bg-[#403F51]"
          />
        </div>

        {/* Card */}
        <Card className="w-[460px] rounded-3xl">
          {step === 1 && <Step1Content />}
          {step === 2 && (
            <Step2Content
              orgName={orgName}
              onOrgNameChange={(v) => { setOrgName(v); setError(null) }}
              slug={slugify(orgName)}
              teamSize={teamSize}
              onTeamSizeChange={setTeamSize}
              error={error}
            />
          )}
          {step === 3 && (
            <Step3Content
              projectName={projectName}
              onProjectNameChange={(v) => { setProjectName(v); setError(null) }}
              description={projectDescription}
              onDescriptionChange={setProjectDescription}
              error={error}
            />
          )}

          <CardFooter className="px-8 pb-8 pt-0 justify-end gap-3">
            {step > 1 && (
              <Button variant="outline" onClick={handleBack} disabled={isSubmitting}>
                Back
              </Button>
            )}
            <Button className="flex-1" onClick={handleContinue} disabled={isSubmitting}>
              {continueLabel}
            </Button>
          </CardFooter>
        </Card>
      </div>
    </div>
  )
}

function Step1Content() {
  return (
    <>
      <CardHeader className="px-8 pt-8 pb-6 gap-2">
        <CardTitle className="text-2xl font-semibold leading-[1.3]">
          Get started with Tompa
        </CardTitle>
        <CardDescription className="text-sm">
          How would you like to set up your workspace?
        </CardDescription>
      </CardHeader>

      <CardContent className="px-8 pb-0 gap-3">
        <RadioGroup name="org-type" value="create">
          {/* Create org — always selected */}
          <div className="flex items-center gap-4 rounded-3xl bg-[#131124] border-2 border-primary p-5">
            <RadioItem value="create" />
            <div className="flex flex-col gap-1 flex-1">
              <span className="text-[15px] font-semibold text-foreground leading-[1.4]">
                Create new organization
              </span>
              <span className="text-[13px] text-muted-foreground leading-relaxed">
                Set up a brand new workspace for your team
              </span>
            </div>
            <CirclePlus className="h-6 w-6 shrink-0 text-primary" />
          </div>

          {/* Join org — disabled */}
          <div className="flex items-center gap-4 rounded-3xl bg-[#1A182E] border border-border p-5 opacity-50 cursor-not-allowed">
            <RadioItem value="join" disabled />
            <div className="flex flex-col gap-1 flex-1">
              <span className="text-[15px] font-semibold text-foreground leading-[1.4]">
                Join existing organization
              </span>
              <span className="text-[13px] text-muted-foreground leading-relaxed">
                Join a workspace you've been invited to
              </span>
            </div>
            <Users className="h-6 w-6 shrink-0 text-muted-foreground" />
          </div>
        </RadioGroup>
      </CardContent>
    </>
  )
}

interface Step2Props {
  orgName: string
  onOrgNameChange: (v: string) => void
  slug: string
  teamSize: string
  onTeamSizeChange: (v: string) => void
  error: string | null
}

function Step2Content({ orgName, onOrgNameChange, slug, teamSize, onTeamSizeChange, error }: Step2Props) {
  return (
    <>
      <CardHeader className="px-8 pt-8 pb-6 gap-2">
        <CardTitle className="text-2xl font-semibold leading-[1.3]">
          Create your organization
        </CardTitle>
        <CardDescription className="text-sm">
          Set up your workspace to get started with Tompa
        </CardDescription>
      </CardHeader>

      <CardContent className="px-8 pb-0 gap-4">
        <InputGroup
          label="Organization name"
          value={orgName}
          onChange={(e) => onOrgNameChange(e.target.value)}
          placeholder="e.g. Acme Inc."
        />
        <InputGroup
          label="URL slug"
          value={slug}
          readOnly
          placeholder="acme-inc"
          className="opacity-60"
        />
        <SelectGroup
          label="Team size"
          value={teamSize}
          onChange={(e) => onTeamSizeChange(e.target.value)}
          placeholder="Select team size"
          options={TEAM_SIZE_OPTIONS}
        />
        {error && <p className="text-sm text-destructive">{error}</p>}
      </CardContent>
    </>
  )
}

interface Step3Props {
  projectName: string
  onProjectNameChange: (v: string) => void
  description: string
  onDescriptionChange: (v: string) => void
  error: string | null
}

function Step3Content({ projectName, onProjectNameChange, description, onDescriptionChange, error }: Step3Props) {
  return (
    <>
      <CardHeader className="px-8 pt-8 pb-6 gap-2">
        <CardTitle className="text-2xl font-semibold leading-[1.3]">
          Create your first project
        </CardTitle>
        <CardDescription className="text-sm">
          Projects help you organize stories and tasks
        </CardDescription>
      </CardHeader>

      <CardContent className="px-8 pb-0 gap-4">
        <InputGroup
          label="Project name"
          value={projectName}
          onChange={(e) => onProjectNameChange(e.target.value)}
          placeholder="e.g. Mobile App Redesign"
        />
        <TextareaGroup
          label="Description (optional)"
          value={description}
          onChange={(e) => onDescriptionChange(e.target.value)}
          placeholder="What is this project about?"
          rows={3}
        />
        {error && <p className="text-sm text-destructive">{error}</p>}
      </CardContent>
    </>
  )
}
