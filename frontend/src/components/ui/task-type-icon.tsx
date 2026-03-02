import { Palette, CircleCheck, Zap } from 'lucide-react'
import { cn } from '@/lib/utils'

export type TaskType = 'design' | 'test' | 'code'

interface TaskTypeIconProps {
  type: TaskType
  className?: string
}

const config: Record<TaskType, { bg: string; color: string; Icon: typeof Palette }> = {
  design: { bg: '#E8D5F0', color: '#7C3AED', Icon: Palette },
  test: { bg: '#D5F0E0', color: '#059669', Icon: CircleCheck },
  code: { bg: '#FEF3C7', color: '#D97706', Icon: Zap },
}

export function TaskTypeIcon({ type, className }: TaskTypeIconProps) {
  const { bg, color, Icon } = config[type]
  return (
    <span
      className={cn(
        'inline-flex shrink-0 items-center justify-center rounded-[4px]',
        className
      )}
      style={{ width: 20, height: 20, background: bg }}
    >
      <Icon size={14} style={{ color }} strokeWidth={2} />
    </span>
  )
}
