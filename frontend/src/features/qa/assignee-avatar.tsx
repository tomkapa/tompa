import { Plus } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Avatar } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { MemberPickerPopover } from './member-picker-popover'
import { useListOrgMembers } from './use-org-members'
import type { OrgMember } from './types'

interface AssigneeAvatarProps {
  assignedTo: string | null
  onAssign: (memberId: string) => void
  onUnassign: () => void
  disabled?: boolean
}

function getInitials(name: string): string {
  return name
    .split(' ')
    .map((w) => w[0])
    .join('')
    .slice(0, 2)
    .toUpperCase()
}

function AssigneeAvatar({ assignedTo, onAssign, onUnassign, disabled = false }: AssigneeAvatarProps) {
  const { data: members = [] } = useListOrgMembers()
  const assignee = assignedTo
    ? (members as OrgMember[]).find((m) => m.user_id === assignedTo) ?? null
    : null

  const trigger = assignee ? (
    <Button
      type="button"
      variant="ghost"
      disabled={disabled}
      title={assignee.display_name}
      aria-label={`Assigned to ${assignee.display_name}. Click to change.`}
      className={cn(
        'h-7 w-7 rounded-full p-0 ring-1 ring-border',
        disabled ? 'cursor-not-allowed' : 'hover:ring-primary',
      )}
    >
      <Avatar
        src={assignee.avatar_url ?? undefined}
        initials={getInitials(assignee.display_name)}
        size="sm"
        className="h-7 w-7 text-[9px]"
      />
    </Button>
  ) : (
    <Button
      type="button"
      variant="outline"
      disabled={disabled}
      aria-label="Assign question to a team member"
      className={cn(
        'h-7 w-7 rounded-full p-0 border-muted-foreground',
        disabled
          ? 'cursor-not-allowed opacity-40'
          : 'text-muted-foreground hover:border-foreground hover:text-foreground',
      )}
    >
      <Plus className="h-3.5 w-3.5" />
    </Button>
  )

  if (disabled) return trigger

  return (
    <MemberPickerPopover
      currentAssigneeId={assignedTo}
      onSelect={(memberId) => {
        if (memberId === null) {
          onUnassign()
        } else {
          onAssign(memberId)
        }
      }}
    >
      {trigger}
    </MemberPickerPopover>
  )
}

export { AssigneeAvatar }
export type { AssigneeAvatarProps }
