import * as React from 'react'
import { Pencil, Check, X, Plus, Trash2 } from 'lucide-react'
import { IconButton } from '@/components/ui/icon-button'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

// ── Text section editor ─────────────────────────────────────────────────────

interface TextSectionEditorProps {
  label: string
  value: string
  onChange: (value: string) => void
}

function TextSectionEditor({ label, value, onChange }: TextSectionEditorProps) {
  const [editing, setEditing] = React.useState(false)
  const [draft, setDraft] = React.useState(value)

  function handleSave() {
    onChange(draft)
    setEditing(false)
  }

  function handleCancel() {
    setDraft(value)
    setEditing(false)
  }

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center gap-2">
        <span className="text-sm font-semibold text-foreground">{label}</span>
        {!editing && (
          <IconButton
            variant="ghost"
            className="h-6 w-6 text-muted-foreground"
            onClick={() => setEditing(true)}
            aria-label={`Edit ${label}`}
          >
            <Pencil className="h-3 w-3" />
          </IconButton>
        )}
      </div>
      {editing ? (
        <div className="flex items-center gap-2">
          <Input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            className="flex-1"
          />
          <IconButton variant="ghost" onClick={handleSave} aria-label="Save">
            <Check className="h-4 w-4 text-emerald-500" />
          </IconButton>
          <IconButton variant="ghost" onClick={handleCancel} aria-label="Cancel">
            <X className="h-4 w-4 text-muted-foreground" />
          </IconButton>
        </div>
      ) : (
        <p className="text-sm text-muted-foreground">{value || 'Not set'}</p>
      )}
    </div>
  )
}

// ── List section editor ─────────────────────────────────────────────────────

interface ListSectionEditorProps {
  label: string
  items: string[]
  onChange: (items: string[]) => void
}

function ListSectionEditor({ label, items, onChange }: ListSectionEditorProps) {
  const [editing, setEditing] = React.useState(false)
  const [draft, setDraft] = React.useState<string[]>(items)
  const [newItem, setNewItem] = React.useState('')

  function handleSave() {
    onChange(draft)
    setEditing(false)
  }

  function handleCancel() {
    setDraft(items)
    setEditing(false)
  }

  function handleAddItem() {
    if (!newItem.trim()) return
    setDraft([...draft, newItem.trim()])
    setNewItem('')
  }

  function handleRemoveItem(index: number) {
    setDraft(draft.filter((_, i) => i !== index))
  }

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center gap-2">
        <span className="text-sm font-semibold text-foreground">{label}</span>
        {!editing && (
          <IconButton
            variant="ghost"
            className="h-6 w-6 text-muted-foreground"
            onClick={() => { setDraft(items); setEditing(true) }}
            aria-label={`Edit ${label}`}
          >
            <Pencil className="h-3 w-3" />
          </IconButton>
        )}
      </div>

      {editing ? (
        <div className="flex flex-col gap-2">
          {draft.map((item, i) => (
            <div key={i} className="flex items-center gap-2">
              <Input
                value={item}
                onChange={(e) => {
                  const next = [...draft]
                  next[i] = e.target.value
                  setDraft(next)
                }}
                className="flex-1"
              />
              <IconButton
                variant="ghost"
                onClick={() => handleRemoveItem(i)}
                aria-label="Remove"
                className="h-6 w-6 text-muted-foreground hover:text-red-400"
              >
                <Trash2 className="h-3 w-3" />
              </IconButton>
            </div>
          ))}
          <div className="flex items-center gap-2">
            <Input
              value={newItem}
              onChange={(e) => setNewItem(e.target.value)}
              placeholder="Add item..."
              className="flex-1"
              onKeyDown={(e) => { if (e.key === 'Enter') handleAddItem() }}
            />
            <IconButton variant="ghost" onClick={handleAddItem} aria-label="Add">
              <Plus className="h-4 w-4 text-emerald-500" />
            </IconButton>
          </div>
          <div className="flex gap-2 pt-1">
            <Button size="default" onClick={handleSave}>Save</Button>
            <Button variant="ghost" onClick={handleCancel}>Cancel</Button>
          </div>
        </div>
      ) : (
        <ul className="flex flex-col gap-1">
          {items.length === 0 && (
            <li className="text-sm text-muted-foreground">None yet</li>
          )}
          {items.map((item, i) => (
            <li key={i} className="text-sm text-muted-foreground">
              {item}
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}

// ── Key-value section editor (for tech_stack) ───────────────────────────────

interface KvSectionEditorProps {
  label: string
  entries: Record<string, string>
  onChange: (entries: Record<string, string>) => void
}

function KvSectionEditor({ label, entries, onChange }: KvSectionEditorProps) {
  const [editing, setEditing] = React.useState(false)
  const [draft, setDraft] = React.useState<[string, string][]>(Object.entries(entries))
  const [newKey, setNewKey] = React.useState('')
  const [newVal, setNewVal] = React.useState('')

  function handleSave() {
    const result: Record<string, string> = {}
    for (const [k, v] of draft) {
      if (k.trim()) result[k.trim()] = v.trim()
    }
    onChange(result)
    setEditing(false)
  }

  function handleCancel() {
    setDraft(Object.entries(entries))
    setEditing(false)
  }

  function handleAdd() {
    if (!newKey.trim()) return
    setDraft([...draft, [newKey.trim(), newVal.trim()]])
    setNewKey('')
    setNewVal('')
  }

  function handleRemove(index: number) {
    setDraft(draft.filter((_, i) => i !== index))
  }

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center gap-2">
        <span className="text-sm font-semibold text-foreground">{label}</span>
        {!editing && (
          <IconButton
            variant="ghost"
            className="h-6 w-6 text-muted-foreground"
            onClick={() => { setDraft(Object.entries(entries)); setEditing(true) }}
            aria-label={`Edit ${label}`}
          >
            <Pencil className="h-3 w-3" />
          </IconButton>
        )}
      </div>

      {editing ? (
        <div className="flex flex-col gap-2">
          {draft.map(([k, v], i) => (
            <div key={i} className="flex items-center gap-2">
              <Input
                value={k}
                onChange={(e) => {
                  const next = [...draft] as [string, string][]
                  next[i] = [e.target.value, v]
                  setDraft(next)
                }}
                placeholder="Key"
                className="w-32"
              />
              <Input
                value={v}
                onChange={(e) => {
                  const next = [...draft] as [string, string][]
                  next[i] = [k, e.target.value]
                  setDraft(next)
                }}
                placeholder="Value"
                className="flex-1"
              />
              <IconButton
                variant="ghost"
                onClick={() => handleRemove(i)}
                aria-label="Remove"
                className="h-6 w-6 text-muted-foreground hover:text-red-400"
              >
                <Trash2 className="h-3 w-3" />
              </IconButton>
            </div>
          ))}
          <div className="flex items-center gap-2">
            <Input
              value={newKey}
              onChange={(e) => setNewKey(e.target.value)}
              placeholder="New key"
              className="w-32"
            />
            <Input
              value={newVal}
              onChange={(e) => setNewVal(e.target.value)}
              placeholder="New value"
              className="flex-1"
              onKeyDown={(e) => { if (e.key === 'Enter') handleAdd() }}
            />
            <IconButton variant="ghost" onClick={handleAdd} aria-label="Add">
              <Plus className="h-4 w-4 text-emerald-500" />
            </IconButton>
          </div>
          <div className="flex gap-2 pt-1">
            <Button size="default" onClick={handleSave}>Save</Button>
            <Button variant="ghost" onClick={handleCancel}>Cancel</Button>
          </div>
        </div>
      ) : (
        <div className="flex flex-col gap-1">
          {Object.keys(entries).length === 0 && (
            <span className="text-sm text-muted-foreground">None yet</span>
          )}
          {Object.entries(entries).map(([k, v]) => (
            <div key={k} className="flex gap-2 text-sm">
              <span className="font-medium text-foreground capitalize">{k}:</span>
              <span className="text-muted-foreground">{v}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export { TextSectionEditor, ListSectionEditor, KvSectionEditor }
