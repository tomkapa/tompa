import * as React from 'react'
import ReactDOM from 'react-dom'
import { Bold, Italic, Heading2, List, ListOrdered, Code, Pencil, Maximize2, Minimize2 } from 'lucide-react'
import { cn } from '@/lib/utils'
import { MarkdownViewer } from '@/components/ui/markdown-viewer'
import { Textarea } from '@/components/ui/textarea'
import { Button } from '@/components/ui/button'

// ── Toolbar helpers ───────────────────────────────────────────────────────────

function applyWrap(
  el: HTMLTextAreaElement,
  draft: string,
  before: string,
  after: string,
  placeholder: string,
  setDraft: (v: string) => void,
) {
  const start = el.selectionStart
  const end = el.selectionEnd
  const selected = draft.slice(start, end) || placeholder
  const next = draft.slice(0, start) + before + selected + after + draft.slice(end)
  setDraft(next)
  requestAnimationFrame(() => {
    el.focus()
    el.setSelectionRange(start + before.length, start + before.length + selected.length)
  })
}

function applyLinePrefix(
  el: HTMLTextAreaElement,
  draft: string,
  prefix: string,
  setDraft: (v: string) => void,
) {
  const start = el.selectionStart
  const lineStart = draft.lastIndexOf('\n', start - 1) + 1
  const next = draft.slice(0, lineStart) + prefix + draft.slice(lineStart)
  setDraft(next)
  requestAnimationFrame(() => {
    el.focus()
    el.setSelectionRange(start + prefix.length, start + prefix.length)
  })
}

// ── Toolbar ───────────────────────────────────────────────────────────────────

interface ToolbarProps {
  textareaRef: React.RefObject<HTMLTextAreaElement | null>
  draft: string
  setDraft: (v: string) => void
}

function ToolbarSeparator() {
  return <div className="mx-1 h-4 w-px shrink-0 bg-border" />
}

function ToolbarBtn({
  title,
  onClick,
  children,
}: {
  title: string
  onClick: () => void
  children: React.ReactNode
}) {
  return (
    <button
      type="button"
      title={title}
      onMouseDown={(e) => {
        // Prevent textarea from losing focus
        e.preventDefault()
        onClick()
      }}
      className="flex h-6 w-6 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
    >
      {children}
    </button>
  )
}

function Toolbar({ textareaRef, draft, setDraft }: ToolbarProps) {
  function act(fn: (el: HTMLTextAreaElement) => void) {
    const el = textareaRef.current
    if (el) fn(el)
  }

  return (
    <div className="flex items-center gap-0.5">
      <ToolbarBtn title="Bold" onClick={() => act((el) => applyWrap(el, draft, '**', '**', 'bold text', setDraft))}>
        <Bold className="h-3.5 w-3.5" />
      </ToolbarBtn>
      <ToolbarBtn title="Italic" onClick={() => act((el) => applyWrap(el, draft, '_', '_', 'italic text', setDraft))}>
        <Italic className="h-3.5 w-3.5" />
      </ToolbarBtn>
      <ToolbarSeparator />
      <ToolbarBtn title="Heading" onClick={() => act((el) => applyLinePrefix(el, draft, '## ', setDraft))}>
        <Heading2 className="h-3.5 w-3.5" />
      </ToolbarBtn>
      <ToolbarSeparator />
      <ToolbarBtn title="Bullet list" onClick={() => act((el) => applyLinePrefix(el, draft, '- ', setDraft))}>
        <List className="h-3.5 w-3.5" />
      </ToolbarBtn>
      <ToolbarBtn title="Numbered list" onClick={() => act((el) => applyLinePrefix(el, draft, '1. ', setDraft))}>
        <ListOrdered className="h-3.5 w-3.5" />
      </ToolbarBtn>
      <ToolbarSeparator />
      <ToolbarBtn title="Inline code" onClick={() => act((el) => applyWrap(el, draft, '`', '`', 'code', setDraft))}>
        <Code className="h-3.5 w-3.5" />
      </ToolbarBtn>
    </div>
  )
}

// ── Fullscreen overlay ────────────────────────────────────────────────────────

interface FullscreenOverlayProps {
  content: string
  placeholder?: string
  readOnly: boolean
  onClose: () => void
  onEdit: () => void
}

function FullscreenOverlay({ content, placeholder, readOnly, onClose, onEdit }: FullscreenOverlayProps) {
  React.useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose()
    }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [onClose])

  const isEmpty = !content || !content.trim()

  return ReactDOM.createPortal(
    <div className="fixed inset-0 z-[60] flex flex-col bg-background/95 backdrop-blur-sm animate-in fade-in-0">
      {/* Header */}
      <div className="flex shrink-0 items-center justify-between border-b border-border px-6 py-3">
        <span className="text-sm font-semibold text-foreground">Description</span>
        <div className="flex items-center gap-2">
          {!readOnly && (
            <Button
              type="button"
              variant="secondary"
              className="h-7 gap-1 px-3 text-xs"
              onClick={() => { onClose(); onEdit() }}
              leadingIcon={<Pencil className="h-3 w-3" />}
            >
              Edit
            </Button>
          )}
          <button
            type="button"
            title="Exit fullscreen (Esc)"
            onClick={onClose}
            className="flex h-7 w-7 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
          >
            <Minimize2 className="h-4 w-4" />
          </button>
        </div>
      </div>

      {/* Scrollable document body */}
      <div className="flex-1 overflow-y-auto">
        <div className="mx-auto w-full max-w-2xl px-8 py-10">
          {isEmpty ? (
            <p className="text-[13px] italic text-muted-foreground/60">
              {placeholder ?? 'No description.'}
            </p>
          ) : (
            <MarkdownViewer content={content} />
          )}
        </div>
      </div>
    </div>,
    document.body,
  )
}

// ── MarkdownEditor ────────────────────────────────────────────────────────────

interface MarkdownEditorProps {
  value: string
  onSave: (v: string) => void
  isSaving?: boolean
  readOnly?: boolean
  placeholder?: string
  className?: string
}

export function MarkdownEditor({
  value,
  onSave,
  isSaving = false,
  readOnly = false,
  placeholder,
  className,
}: MarkdownEditorProps) {
  const [editing, setEditing] = React.useState(false)
  const [preview, setPreview] = React.useState(false)
  const [fullscreen, setFullscreen] = React.useState(false)
  const [draft, setDraft] = React.useState(value)
  const textareaRef = React.useRef<HTMLTextAreaElement>(null)

  // Sync draft when parent value changes while not editing (SSE-driven updates)
  React.useEffect(() => {
    if (!editing) setDraft(value)
  }, [value, editing])

  function startEditing() {
    setDraft(value)
    setPreview(false)
    setEditing(true)
  }

  function save() {
    setEditing(false)
    setPreview(false)
    if (draft !== value) onSave(draft)
  }

  function cancel() {
    setDraft(value)
    setEditing(false)
    setPreview(false)
  }

  // ── View mode ─────────────────────────────────────────────────────────────
  if (!editing) {
    const isEmpty = !value || !value.trim()
    return (
      <>
        {fullscreen && (
          <FullscreenOverlay
            content={value}
            placeholder={placeholder}
            readOnly={readOnly}
            onClose={() => setFullscreen(false)}
            onEdit={startEditing}
          />
        )}
        <div className={cn('group relative', className)}>
          {isEmpty ? (
            <p className="text-[13px] italic text-muted-foreground/60">
              {placeholder ?? 'No description.'}
            </p>
          ) : (
            <MarkdownViewer content={value} />
          )}
          {/* Action buttons — visible on hover */}
          <div className="absolute right-0 top-0 flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
            <button
              type="button"
              title="Fullscreen"
              onClick={() => setFullscreen(true)}
              className="flex h-6 w-6 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
            >
              <Maximize2 className="h-3 w-3" />
            </button>
            {!readOnly && (
              <Button
                type="button"
                variant="ghost"
                className="h-7 gap-1 px-2 text-[11px] text-muted-foreground"
                onClick={startEditing}
                leadingIcon={<Pencil className="h-3 w-3" />}
              >
                Edit
              </Button>
            )}
          </div>
        </div>
      </>
    )
  }

  // ── Edit mode ─────────────────────────────────────────────────────────────
  return (
    <div className={cn('flex flex-col overflow-hidden rounded-[16px] border border-primary/50', className)}>
      {/* Top bar: Edit/Preview tabs + toolbar */}
      <div className="flex items-center gap-3 border-b border-border bg-muted/30 px-3 py-1.5">
        <div className="flex items-center rounded-md bg-muted p-0.5">
          <button
            type="button"
            onClick={() => setPreview(false)}
            className={cn(
              'rounded px-2.5 py-0.5 text-xs font-medium transition-colors',
              !preview
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground',
            )}
          >
            Edit
          </button>
          <button
            type="button"
            onClick={() => setPreview(true)}
            className={cn(
              'rounded px-2.5 py-0.5 text-xs font-medium transition-colors',
              preview
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground',
            )}
          >
            Preview
          </button>
        </div>

        {!preview && (
          <>
            <div className="h-4 w-px bg-border" />
            <Toolbar textareaRef={textareaRef} draft={draft} setDraft={setDraft} />
          </>
        )}
      </div>

      {/* Content */}
      {preview ? (
        <div className="min-h-32 px-4 py-3">
          {draft.trim() ? (
            <MarkdownViewer content={draft} />
          ) : (
            <p className="text-[13px] italic text-muted-foreground/60">
              {placeholder ?? 'Nothing to preview.'}
            </p>
          )}
        </div>
      ) : (
        <Textarea
          ref={textareaRef}
          className="min-h-32 rounded-none border-none bg-transparent px-4 py-3 text-[13px] focus-visible:ring-0"
          value={draft}
          autoFocus
          disabled={isSaving}
          placeholder={placeholder}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && e.ctrlKey) {
              e.preventDefault()
              save()
            } else if (e.key === 'Escape') {
              e.preventDefault()
              cancel()
            }
          }}
        />
      )}

      {/* Footer */}
      <div className="flex items-center gap-2 border-t border-border bg-muted/30 px-3 py-2">
        <span className="mr-auto text-[11px] text-muted-foreground">
          Ctrl+Enter to save · Esc to cancel
        </span>
        <Button type="button" variant="secondary" className="h-7 px-3 text-xs" onClick={cancel} disabled={isSaving}>
          Cancel
        </Button>
        <Button type="button" className="h-7 px-3 text-xs" onClick={save} disabled={isSaving}>
          {isSaving ? 'Saving…' : 'Save'}
        </Button>
      </div>
    </div>
  )
}
