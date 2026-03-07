import * as React from 'react'
import { ArrowLeft, RefreshCw, Check, Sparkles, Pencil, X } from 'lucide-react'
import { MarkdownViewer } from '@/components/ui/markdown-viewer'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { IconButton } from '@/components/ui/icon-button'
import { Input } from '@/components/ui/input'
import { Textarea, TextareaGroup } from '@/components/ui/textarea'
import { ListboxGroup } from '@/components/ui/listbox'
import {
  Dialog,
  DialogPortal,
  DialogOverlay,
  DialogClose,
} from '@/components/ui/dialog'

export type StoryType = 'feature' | 'bug' | 'refactor'

export interface StoryFormData {
  title: string
  description: string
  ownerId: string
  storyType: StoryType
}

export interface StoryCreationProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  owners: { id: string; name: string }[]
  expandedDescription?: string
  isGenerating?: boolean
  onRequestExpansion: (data: StoryFormData) => void
  onRegenerate?: (data: StoryFormData) => void
  onApprove: (data: StoryFormData, editedDescription: string) => void
}

const STORY_TYPE_OPTIONS: { value: StoryType; label: string }[] = [
  { value: 'feature', label: 'Feature' },
  { value: 'bug', label: 'Bug' },
  { value: 'refactor', label: 'Refactor' },
]

const storyTypeBadge: Record<StoryType, { label: string; className: string }> = {
  feature: {
    label: 'Feature',
    className: 'bg-[var(--color-success)] text-[var(--color-success-foreground)]',
  },
  bug: {
    label: 'Bug',
    className: 'bg-destructive text-destructive-foreground',
  },
  refactor: {
    label: 'Refactor',
    className: 'bg-[var(--color-info)] text-[var(--color-info-foreground)]',
  },
}

// ── Step 1: Input Form ────────────────────────────────────────────────────────

interface InputStepProps {
  formData: StoryFormData
  owners: { id: string; name: string }[]
  isLoading: boolean
  onChange: (data: Partial<StoryFormData>) => void
  onSubmit: () => void
  onCancel: () => void
}

function InputStep({ formData, owners, isLoading, onChange, onSubmit, onCancel }: InputStepProps) {
  return (
    <>
      {/* Header */}
      <div className="border-b border-border px-8 py-6 flex flex-col gap-2">
        <h2 className="text-xl font-semibold text-card-foreground leading-[1.4]">
          Create New Story
        </h2>
        <p className="text-sm text-muted-foreground leading-[1.4]">
          Add a brief description and the AI will expand it into a structured story.
        </p>
      </div>

      {/* Content */}
      <div className="flex flex-col gap-5 px-8 py-6">
        <Input
          label="Title"
          placeholder="e.g., User authentication flow"
          value={formData.title}
          onChange={(e) => onChange({ title: e.target.value })}
        />

        <TextareaGroup
          label="Brief Description"
          placeholder="Describe what this story should accomplish in 1-2 sentences..."
          value={formData.description}
          onChange={(e) => onChange({ description: e.target.value })}
          rows={3}
        />

        <ListboxGroup
          label="Owner"
          placeholder="Select owner"
          value={formData.ownerId}
          onChange={(ownerId) => onChange({ ownerId })}
          options={owners.map((o) => ({ value: o.id, label: o.name }))}
        />

        {/* Story Type */}
        <div className="flex flex-col gap-2">
          <label className="text-sm font-medium text-foreground leading-[1.4]">Story Type</label>
          <div className="flex gap-3">
            {STORY_TYPE_OPTIONS.map((opt) => (
              <Button
                key={opt.value}
                type="button"
                variant={formData.storyType === opt.value ? 'default' : 'ghost'}
                onClick={() => onChange({ storyType: opt.value })}
                className={cn(
                  'flex-1 rounded-2xl h-auto py-3',
                  formData.storyType !== opt.value && 'border border-border',
                )}
              >
                {opt.label}
              </Button>
            ))}
          </div>
        </div>
      </div>

      {/* Footer */}
      <div className="flex items-center justify-end gap-3 border-t border-border bg-card px-8 py-5">
        <Button variant="secondary" onClick={onCancel} disabled={isLoading}>
          Cancel
        </Button>
        <Button
          onClick={onSubmit}
          disabled={isLoading || !formData.title.trim()}
        >
          {isLoading ? 'Generating…' : 'Create Story'}
        </Button>
      </div>
    </>
  )
}

// ── Step 2: Review ────────────────────────────────────────────────────────────

interface ReviewStepProps {
  formData: StoryFormData
  expandedDescription: string
  owners: { id: string; name: string }[]
  onBack: () => void
  onRegenerate?: () => void
  onApprove: (editedDescription: string) => void
}

function ReviewStep({
  formData,
  expandedDescription,
  owners,
  onBack,
  onRegenerate,
  onApprove,
}: ReviewStepProps) {
  const [editedDesc, setEditedDesc] = React.useState(expandedDescription)
  const [isEditing, setIsEditing] = React.useState(false)

  React.useEffect(() => {
    setEditedDesc(expandedDescription)
  }, [expandedDescription])

  const ownerName = owners.find((o) => o.id === formData.ownerId)?.name ?? formData.ownerId
  const badge = storyTypeBadge[formData.storyType]

  return (
    <>
      {/* Header */}
      <div className="border-b border-border px-8 py-6 flex flex-col gap-2">
        <div className="flex items-center gap-3">
          <h2 className="text-xl font-semibold text-card-foreground leading-[1.4]">
            Review AI-Expanded Story
          </h2>
          <span className="inline-flex items-center gap-1 rounded-full bg-[var(--color-info)] px-2 py-1">
            <Sparkles className="h-3 w-3 text-[var(--color-info-foreground)]" />
            <span className="text-[11px] font-medium text-[var(--color-info-foreground)]">
              AI Generated
            </span>
          </span>
        </div>
        <p className="text-sm text-muted-foreground leading-[1.4]">
          Review and edit the expanded description before creating the story.
        </p>
      </div>

      {/* Content */}
      <div className="flex flex-col gap-5 px-8 py-6">
        {/* Story info row */}
        <div className="flex items-center gap-3">
          <span className="text-base font-semibold text-foreground leading-[1.3]">
            {formData.title}
          </span>
          <span
            className={cn(
              'inline-flex items-center rounded-full px-[10px] py-1 text-xs font-medium leading-[1.2] whitespace-nowrap',
              badge.className
            )}
          >
            {badge.label}
          </span>
        </div>

        {/* Expanded description */}
        <div className="flex flex-col gap-3 rounded-lg border border-border bg-accent p-4">
          <span className="text-xs font-medium text-muted-foreground leading-[1.3]">
            Expanded Description
          </span>

          {isEditing ? (
            <Textarea
              className="min-h-32 bg-transparent border-none rounded-none px-0 py-0 focus-visible:ring-0"
              value={editedDesc}
              onChange={(e) => setEditedDesc(e.target.value)}
              onBlur={() => setIsEditing(false)}
              autoFocus
            />
          ) : (
            <button
              type="button"
              onClick={() => setIsEditing(true)}
              className="w-full text-left"
            >
              <MarkdownViewer content={editedDesc} />
            </button>
          )}

          <div className="flex items-center gap-1">
            <Pencil className="h-3 w-3 text-muted-foreground" />
            <span className="text-[11px] text-muted-foreground">Click to edit</span>
          </div>
        </div>

        {/* Owner row */}
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Owner:</span>
          <span className="text-sm font-medium text-foreground">{ownerName}</span>
        </div>
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between gap-3 border-t border-border bg-card px-8 py-5">
        <Button variant="secondary" onClick={onBack} leadingIcon={<ArrowLeft className="h-4 w-4" />}>
          Back
        </Button>
        <div className="flex items-center gap-3">
          {onRegenerate && (
            <Button
              variant="secondary"
              onClick={onRegenerate}
              leadingIcon={<RefreshCw className="h-4 w-4" />}
            >
              Regenerate
            </Button>
          )}
          <Button
            onClick={() => onApprove(editedDesc)}
            leadingIcon={<Check className="h-4 w-4" />}
          >
            Approve &amp; Create
          </Button>
        </div>
      </div>
    </>
  )
}

// ── Main Component ────────────────────────────────────────────────────────────

const DEFAULT_FORM: StoryFormData = {
  title: '',
  description: '',
  ownerId: '',
  storyType: 'feature',
}

export function StoryCreation({
  open,
  onOpenChange,
  owners,
  expandedDescription,
  isGenerating = false,
  onRequestExpansion,
  onRegenerate,
  onApprove,
}: StoryCreationProps) {
  const [formData, setFormData] = React.useState<StoryFormData>(DEFAULT_FORM)
  const step: 'input' | 'review' =
    expandedDescription !== undefined && !isGenerating ? 'review' : 'input'

  function handleClose() {
    onOpenChange(false)
    setFormData(DEFAULT_FORM)
  }

  function handleChange(partial: Partial<StoryFormData>) {
    setFormData((prev) => ({ ...prev, ...partial }))
  }

  function handleSubmit() {
    onRequestExpansion(formData)
  }

  function handleBack() {
    onRequestExpansion({ ...formData, title: '' }) // signal parent to clear expansion
    // In practice the parent should clear expandedDescription when going back
    setFormData((prev) => prev)
  }

  function handleApprove(editedDescription: string) {
    onApprove(formData, editedDescription)
    handleClose()
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogPortal>
        <DialogOverlay />
        <div
          role="dialog"
          aria-modal
          className="fixed left-1/2 top-1/2 z-50 w-full max-w-[520px] -translate-x-1/2 -translate-y-1/2 rounded-[24px] bg-card border border-border shadow-xl overflow-hidden animate-in fade-in-0 zoom-in-95"
          onClick={(e) => e.stopPropagation()}
        >
          <DialogClose asChild>
            <IconButton
              type="button"
              variant="ghost"
              aria-label="Close"
              className="absolute right-6 top-6 h-7 w-7 text-muted-foreground z-10"
            >
              <X className="h-4 w-4" />
            </IconButton>
          </DialogClose>

          {step === 'input' ? (
            <InputStep
              formData={formData}
              owners={owners}
              isLoading={isGenerating}
              onChange={handleChange}
              onSubmit={handleSubmit}
              onCancel={handleClose}
            />
          ) : (
            <ReviewStep
              formData={formData}
              expandedDescription={expandedDescription!}
              owners={owners}
              onBack={handleBack}
              onRegenerate={onRegenerate ? () => onRegenerate(formData) : undefined}
              onApprove={handleApprove}
            />
          )}
        </div>
      </DialogPortal>
    </Dialog>
  )
}
