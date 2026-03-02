import { ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils'
import { DecisionTrailEntry } from '@/components/ui/decision-trail-entry'

type DecisionStage =
  | 'grooming'
  | 'planning'
  | 'task-decomposition'
  | 'per-task-qa'
  | 'per-task-impl'
  | 'task-qa'
  | 'impl'

interface Decision {
  id: string
  domain: string
  questionText: string
  answerText: string
  superseded: boolean
  stage: DecisionStage
  entryUrl?: string
}

interface DecisionTrailProps {
  decisions: Decision[]
  level: 'story' | 'task'
  className?: string
}

const STAGE_LABELS: Record<DecisionStage, string> = {
  grooming: 'Grooming',
  planning: 'Planning',
  'task-decomposition': 'Task Decomposition',
  'per-task-qa': 'Per-task Q&A',
  'per-task-impl': 'Per-task Implementation',
  'task-qa': 'Task Q&A',
  impl: 'Implementation Decisions',
}

const STORY_STAGE_ORDER: DecisionStage[] = [
  'grooming',
  'planning',
  'task-decomposition',
  'per-task-qa',
  'per-task-impl',
]

const TASK_STAGE_ORDER: DecisionStage[] = ['task-qa', 'impl']

function DecisionTrail({ decisions, level, className }: DecisionTrailProps) {
  const stageOrder = level === 'story' ? STORY_STAGE_ORDER : TASK_STAGE_ORDER

  const grouped = stageOrder.reduce<Record<string, Decision[]>>((acc, stage) => {
    const entries = decisions.filter((d) => d.stage === stage)
    if (entries.length > 0) {
      acc[stage] = entries
    }
    return acc
  }, {})

  const totalCount = decisions.length

  return (
    <div
      className={cn(
        'flex flex-col overflow-hidden rounded-[24px] border border-border bg-background',
        className
      )}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-4 border-b border-border">
        <span className="text-[16px] font-semibold leading-[1.2] text-foreground">
          Decision Trail
        </span>
        <span className="text-[13px] font-normal leading-[1.2] text-muted-foreground">
          {totalCount} decision{totalCount !== 1 ? 's' : ''}
        </span>
      </div>

      {/* Scrollable content */}
      <div className="flex flex-col gap-4 overflow-y-auto p-5">
        {Object.entries(grouped).map(([stage, entries]) => (
          <div key={stage} className="flex flex-col gap-3">
            {/* Stage header */}
            <div className="flex items-center gap-2">
              <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
              <span className="text-[13px] font-semibold leading-[1.2] text-foreground">
                {STAGE_LABELS[stage as DecisionStage]}
              </span>
              <span className="text-[12px] font-normal leading-[1.2] text-muted-foreground">
                {entries.length} decision{entries.length !== 1 ? 's' : ''}
              </span>
            </div>

            {/* Stage entries */}
            <div className="flex flex-col gap-2 pl-6">
              {entries.map((decision) => (
                <DecisionTrailEntry
                  key={decision.id}
                  domain={decision.domain}
                  questionText={decision.questionText}
                  answerText={decision.answerText}
                  superseded={decision.superseded}
                  entryUrl={decision.entryUrl}
                />
              ))}
            </div>
          </div>
        ))}

        {Object.keys(grouped).length === 0 && (
          <p className="text-[13px] text-muted-foreground">No decisions yet.</p>
        )}
      </div>
    </div>
  )
}

export { DecisionTrail }
export type { Decision, DecisionStage, DecisionTrailProps }
