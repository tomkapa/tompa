import * as React from 'react'
import * as ReactDOM from 'react-dom'
import { Check, UserX, Search } from 'lucide-react'
import { Avatar } from '@/components/ui/avatar'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { useListOrgMembers } from './use-org-members'
import type { OrgMember } from './types'

interface MemberPickerPopoverProps {
  currentAssigneeId: string | null
  onSelect: (memberId: string | null) => void
  children: React.ReactNode
}

function getInitials(name: string): string {
  return name
    .split(' ')
    .map((w) => w[0])
    .join('')
    .slice(0, 2)
    .toUpperCase()
}

function MemberPickerPopover({ currentAssigneeId, onSelect, children }: MemberPickerPopoverProps) {
  const [open, setOpen] = React.useState(false)
  const [search, setSearch] = React.useState('')
  const triggerRef = React.useRef<HTMLDivElement>(null)
  const popoverRef = React.useRef<HTMLDivElement>(null)
  const [position, setPosition] = React.useState({ top: 0, left: 0 })

  const { data: members = [] } = useListOrgMembers()

  const filtered = React.useMemo(() => {
    const q = search.toLowerCase()
    return (members as OrgMember[]).filter((m) =>
      m.display_name.toLowerCase().includes(q),
    )
  }, [members, search])

  function updatePosition() {
    if (!triggerRef.current) return
    // display:contents has no box — use first child element's rect instead
    const el = (triggerRef.current.firstElementChild as HTMLElement) ?? triggerRef.current
    const rect = el.getBoundingClientRect()
    // position:fixed is viewport-relative; do NOT add window scroll offsets
    setPosition({
      top: rect.bottom + 4,
      left: rect.right - 240,
    })
  }

  function handleOpen() {
    updatePosition()
    setOpen(true)
    setSearch('')
  }

  function handleSelect(memberId: string | null) {
    onSelect(memberId)
    setOpen(false)
  }

  // Close on outside click or Escape
  React.useEffect(() => {
    if (!open) return
    function onPointerDown(e: PointerEvent) {
      if (
        triggerRef.current?.contains(e.target as Node) ||
        popoverRef.current?.contains(e.target as Node)
      )
        return
      setOpen(false)
    }
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') setOpen(false)
    }
    document.addEventListener('pointerdown', onPointerDown)
    document.addEventListener('keydown', onKeyDown)
    return () => {
      document.removeEventListener('pointerdown', onPointerDown)
      document.removeEventListener('keydown', onKeyDown)
    }
  }, [open])

  const popover = open
    ? ReactDOM.createPortal(
        <div
          ref={popoverRef}
          style={{ top: position.top, left: position.left, width: 240 }}
          className="fixed z-[200] flex flex-col overflow-hidden rounded-[var(--radius-xs)] border border-border bg-popover shadow-[0_4px_16px_-2px_rgba(0,0,0,0.2)]"
        >
          {/* Search */}
          <div className="flex items-center gap-2 border-b border-border px-3 py-2">
            <Search className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
            <Input
              autoFocus
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search members…"
              className="flex-1 border-none bg-transparent rounded-none px-0 py-0 focus-visible:ring-0 text-sm"
            />
          </div>

          {/* Remove assignment */}
          {currentAssigneeId && (
            <>
              <Button
                type="button"
                variant="ghost"
                onClick={() => handleSelect(null)}
                className="w-full justify-start rounded-full px-4 py-2.5 h-auto text-sm text-muted-foreground hover:text-foreground"
                leadingIcon={<UserX className="h-4 w-4 shrink-0" />}
              >
                Remove assignment
              </Button>
              <div className="mx-3 h-px bg-border" />
            </>
          )}

          {/* Member list */}
          <div className="flex flex-col py-1">
            {filtered.length === 0 ? (
              <p className="px-4 py-2.5 text-sm text-muted-foreground">No members found</p>
            ) : (
              filtered.map((member) => {
                const isSelected = member.user_id === currentAssigneeId
                return (
                  <Button
                    key={member.user_id}
                    type="button"
                    variant="ghost"
                    onClick={() => handleSelect(member.user_id)}
                    className="w-full justify-start rounded-full px-4 py-2 h-auto text-sm text-popover-foreground"
                  >
                    <Avatar
                      src={member.avatar_url ?? undefined}
                      initials={getInitials(member.display_name)}
                      size="sm"
                      className="h-6 w-6 text-[9px]"
                    />
                    <span className="flex-1 truncate text-left">{member.display_name}</span>
                    {isSelected && (
                      <Check className="h-4 w-4 shrink-0 text-primary" />
                    )}
                  </Button>
                )
              })
            )}
          </div>
        </div>,
        document.body,
      )
    : null

  return (
    <>
      <div ref={triggerRef} onClick={handleOpen} style={{ display: 'contents' }}>
        {children}
      </div>
      {popover}
    </>
  )
}

export { MemberPickerPopover }
export type { MemberPickerPopoverProps }
