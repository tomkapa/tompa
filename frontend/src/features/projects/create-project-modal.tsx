import * as React from 'react'
import { Button } from '@/components/ui/button'
import { InputGroup } from '@/components/ui/input'
import { TextareaGroup } from '@/components/ui/textarea'
import {
  Dialog,
  DialogPortal,
  DialogOverlay,
} from '@/components/ui/dialog'

export interface CreateProjectFormData {
  name: string
  description: string
  githubRepoUrl: string
}

interface CreateProjectModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSubmit: (data: CreateProjectFormData) => void
  isLoading?: boolean
}

const DEFAULT_FORM: CreateProjectFormData = {
  name: '',
  description: '',
  githubRepoUrl: '',
}

export function CreateProjectModal({
  open,
  onOpenChange,
  onSubmit,
  isLoading = false,
}: CreateProjectModalProps) {
  const [formData, setFormData] = React.useState<CreateProjectFormData>(DEFAULT_FORM)

  function handleClose() {
    onOpenChange(false)
    setFormData(DEFAULT_FORM)
  }

  function handleChange(partial: Partial<CreateProjectFormData>) {
    setFormData((prev) => ({ ...prev, ...partial }))
  }

  function handleSubmit() {
    if (!formData.name.trim()) return
    onSubmit(formData)
  }

  // Reset form when opening
  React.useEffect(() => {
    if (open) setFormData(DEFAULT_FORM)
  }, [open])

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
          {/* Header */}
          <div className="border-b border-border px-8 py-6 flex flex-col gap-2">
            <h2 className="text-xl font-semibold text-card-foreground leading-[1.4]">
              Create New Project
            </h2>
            <p className="text-sm text-muted-foreground leading-[1.4]">
              Set up a new project to organize your stories and connect your repository.
            </p>
          </div>

          {/* Content */}
          <div className="flex flex-col gap-5 px-8 py-6">
            <InputGroup
              label="Project Name"
              placeholder="e.g., My Startup App"
              value={formData.name}
              onChange={(e) => handleChange({ name: e.target.value })}
            />

            <TextareaGroup
              label="Description (optional)"
              placeholder="Brief description of this project..."
              value={formData.description}
              onChange={(e) => handleChange({ description: e.target.value })}
              rows={3}
            />

            <InputGroup
              label="Repository URL"
              placeholder="https://github.com/org/repo"
              value={formData.githubRepoUrl}
              onChange={(e) => handleChange({ githubRepoUrl: e.target.value })}
            />
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-3 border-t border-border bg-card px-8 py-5">
            <Button variant="outline" onClick={handleClose} disabled={isLoading}>
              Cancel
            </Button>
            <Button
              onClick={handleSubmit}
              disabled={isLoading || !formData.name.trim()}
            >
              {isLoading ? 'Creating...' : 'Create Project'}
            </Button>
          </div>
        </div>
      </DialogPortal>
    </Dialog>
  )
}
