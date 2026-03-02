import * as React from 'react'
import { X } from 'lucide-react'
import { useParams, useNavigate } from '@tanstack/react-router'
import { useQueryClient } from '@tanstack/react-query'
import { cn } from '@/lib/utils'
import { AppBreadcrumb } from '@/components/ui/app-breadcrumb'
import { TabSwitcher } from '@/components/ui/tab-switcher'
import { ConfirmationDialog } from '@/components/ui/confirmation-dialog'
import { StoryOverview } from '@/features/stories/story-overview'
import { TaskOverview } from '@/features/tasks/task-overview'
import { QaThread } from '@/features/qa/qa-thread'
import { DecisionTrail } from '@/features/decisions/decision-trail'
import type { Decision, DecisionStage } from '@/features/decisions/decision-trail'
import type { QaRound } from '@/features/qa/types'
import { useGetStory } from '@/api/generated/stories/stories'
import { useGetTask, useMarkDone, getGetTaskQueryKey } from '@/api/generated/tasks/tasks'
import {
  useListRounds,
  useSubmitAnswer,
  useCourseCorrect,
  useRollback,
  getListRoundsQueryKey,
} from '@/api/generated/qa/qa'
import type {
  QaRoundResponse,
  QaQuestion as ApiQaQuestion,
  StoryResponse,
} from '@/api/generated/tompaAPI.schemas'

// ── Data mapping ─────────────────────────────────────────────────────────────

const VALID_STAGES = new Set<string>([
  'grooming',
  'planning',
  'task-decomposition',
  'per-task-qa',
  'per-task-impl',
  'task-qa',
  'impl',
])

function toDecisionStage(stage: string): DecisionStage {
  return VALID_STAGES.has(stage) ? (stage as DecisionStage) : 'grooming'
}

function mapApiRound(r: QaRoundResponse): QaRound {
  return {
    id: r.id,
    roundNumber: r.round_number,
    questions: r.content.questions.map((q: ApiQaQuestion) => ({
      id: q.id,
      domain: q.domain,
      text: q.text,
      options: q.options,
      answeredIndex: q.selected_answer_index ?? undefined,
      answeredText: q.selected_answer_text ?? undefined,
    })),
  }
}

function roundsToDecisions(rounds: QaRoundResponse[]): Decision[] {
  const decisions: Decision[] = []
  for (const round of rounds) {
    for (const q of round.content.questions) {
      if (q.selected_answer_index != null || q.selected_answer_text) {
        const answerText =
          q.selected_answer_text ??
          q.options[q.selected_answer_index ?? 0] ??
          ''
        decisions.push({
          id: q.id,
          domain: q.domain,
          questionText: q.text,
          answerText,
          superseded: false,
          stage: toDecisionStage(round.stage),
        })
      }
    }
  }
  return decisions
}

function hasPendingQuestions(rounds: QaRoundResponse[]): boolean {
  if (rounds.length === 0) return false
  const latest = rounds[rounds.length - 1]
  return latest.content.questions.some(
    (q) => q.selected_answer_index == null && !q.selected_answer_text
  )
}

// ── Modal shell ───────────────────────────────────────────────────────────────

interface ModalShellProps {
  breadcrumb: React.ReactNode
  onCloseAttempt: () => void
  children: React.ReactNode
  className?: string
}

function ModalShell({ breadcrumb, onCloseAttempt, children, className }: ModalShellProps) {
  // Close on Escape key
  React.useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onCloseAttempt()
    }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [onCloseAttempt])

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/80"
      aria-modal
      role="dialog"
    >
      <div
        className={cn(
          'flex w-[min(1152px,92vw)] flex-col overflow-hidden rounded-2xl bg-card shadow-[0_16px_48px_rgba(0,0,0,0.2)]',
          'h-[min(720px,90vh)]',
          className
        )}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex shrink-0 items-center justify-between border-b border-border px-6 py-4">
          {breadcrumb}
          <button
            type="button"
            aria-label="Close"
            className="flex h-8 w-8 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
            onClick={onCloseAttempt}
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Two-column content area */}
        <div className="flex min-h-0 flex-1 overflow-hidden">
          {children}
        </div>
      </div>
    </div>
  )
}

// ── Tab panel helper ──────────────────────────────────────────────────────────

const QA_TABS = [
  { id: 'qa', label: 'Q&A Thread' },
  { id: 'decisions', label: 'Decision Trail' },
]

interface RightPanelProps {
  rounds: QaRound[]
  decisions: Decision[]
  level: 'story' | 'task'
  storyId: string
  taskId?: string
  currentStage: string
  onAnswer: (questionId: string, answerIndex: number | null, answerText: string | null) => void
  onRollback: (roundId: string) => void
  onCourseCorrect: (text: string) => void
}

function RightPanel({
  rounds,
  decisions,
  level,
  onAnswer,
  onRollback,
  onCourseCorrect,
}: RightPanelProps) {
  const [activeTab, setActiveTab] = React.useState('qa')

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden p-4">
      <div className="mb-4 flex shrink-0 justify-end">
        <TabSwitcher tabs={QA_TABS} activeId={activeTab} onChange={setActiveTab} />
      </div>
      <div className="min-h-0 flex-1 overflow-hidden">
        {activeTab === 'qa' ? (
          <QaThread
            rounds={rounds}
            onAnswer={onAnswer}
            onRollback={onRollback}
            onCourseCorrect={onCourseCorrect}
          />
        ) : (
          <DecisionTrail decisions={decisions} level={level} />
        )}
      </div>
    </div>
  )
}

// ── Story View (U28) ─────────────────────────────────────────────────────────

interface StoryViewProps {
  projectId: string
  storyId: string
  story: StoryResponse
  rounds: QaRoundResponse[]
  onAnswer: (questionId: string, answerIndex: number | null, answerText: string | null) => void
  onRollback: (roundId: string) => void
  onCourseCorrect: (text: string) => void
  onTaskClick: (taskId: string) => void
}

function StoryViewContent({
  story,
  rounds,
  onAnswer,
  onRollback,
  onCourseCorrect,
  onTaskClick,
}: Omit<StoryViewProps, 'projectId' | 'storyId'>) {
  const mappedRounds = React.useMemo(() => rounds.map(mapApiRound), [rounds])
  const decisions = React.useMemo(() => roundsToDecisions(rounds), [rounds])
  const currentStage = rounds[rounds.length - 1]?.stage ?? 'grooming'

  return (
    <>
      {/* Left column — 40% */}
      <div className="w-[40%] shrink-0 overflow-y-auto border-r border-border p-4">
        <StoryOverview
          story={story}
          tasks={story.tasks}
          onTaskClick={onTaskClick}
          className="h-full"
        />
      </div>

      {/* Right column — 60% */}
      <RightPanel
        rounds={mappedRounds}
        decisions={decisions}
        level="story"
        storyId={story.id}
        currentStage={currentStage}
        onAnswer={onAnswer}
        onRollback={onRollback}
        onCourseCorrect={onCourseCorrect}
      />
    </>
  )
}

// ── Task View (U29) ──────────────────────────────────────────────────────────

interface TaskViewContentProps {
  taskId: string
  storyId: string
  rounds: QaRoundResponse[]
  onAnswer: (questionId: string, answerIndex: number | null, answerText: string | null) => void
  onRollback: (roundId: string) => void
  onCourseCorrect: (text: string) => void
}

function TaskViewContent({
  taskId,
  rounds,
  onAnswer,
  onRollback,
  onCourseCorrect,
}: TaskViewContentProps) {
  const queryClient = useQueryClient()
  const { data: taskResp } = useGetTask(taskId)
  const task = taskResp?.status === 200 ? taskResp.data : null

  const markDoneMutation = useMarkDone({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: getGetTaskQueryKey(taskId) })
      },
    },
  })

  const mappedRounds = React.useMemo(() => rounds.map(mapApiRound), [rounds])
  const decisions = React.useMemo(() => roundsToDecisions(rounds), [rounds])
  const currentStage = rounds[rounds.length - 1]?.stage ?? 'task-qa'

  if (!task) {
    return (
      <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
        Loading task…
      </div>
    )
  }

  return (
    <>
      {/* Left column — 40% */}
      <div className="w-[40%] shrink-0 overflow-y-auto border-r border-border p-4">
        <TaskOverview
          task={task}
          onMarkDone={() => markDoneMutation.mutate({ id: taskId })}
          markDoneLoading={markDoneMutation.isPending}
          className="h-full"
        />
      </div>

      {/* Right column — 60% */}
      <RightPanel
        rounds={mappedRounds}
        decisions={decisions}
        level="task"
        storyId={task.story_id}
        taskId={taskId}
        currentStage={currentStage}
        onAnswer={onAnswer}
        onRollback={onRollback}
        onCourseCorrect={onCourseCorrect}
      />
    </>
  )
}

// ── StoryModal — URL-driven entry point (U28 + U29) ───────────────────────────

/**
 * U28 / U29 — Story Detail Modal.
 *
 * Reads `storyId` (and optionally `taskId`) from the URL via TanStack Router
 * params.  Shows the **Story View** when only `storyId` is present, and the
 * **Task View** when both `storyId` and `taskId` are present.
 *
 * URL patterns:
 *   /projects/:projectId/stories/:storyId          → Story View
 *   /projects/:projectId/stories/:storyId/tasks/:taskId → Task View
 */
export function StoryModal() {
  const allParams = useParams({ strict: false }) as Record<string, string | undefined>
  const projectId = allParams.projectId ?? ''
  const storyId = allParams.storyId ?? ''
  const taskId = allParams.taskId
  const isTaskView = !!taskId

  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const [confirmOpen, setConfirmOpen] = React.useState(false)

  // ── Fetch story ────────────────────────────────────────────────────────────
  const { data: storyResp } = useGetStory(storyId, { query: { enabled: !!storyId } })
  const story = storyResp?.status === 200 ? storyResp.data : null

  // ── Fetch QA rounds ────────────────────────────────────────────────────────
  const roundsParams = isTaskView
    ? { task_id: taskId }
    : { story_id: storyId }

  const { data: roundsResp } = useListRounds(roundsParams, {
    query: { enabled: !!storyId },
  })
  const apiRounds: QaRoundResponse[] = roundsResp?.status === 200 ? roundsResp.data : []

  // ── Mutations ──────────────────────────────────────────────────────────────
  const submitAnswerMutation = useSubmitAnswer({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({
          queryKey: getListRoundsQueryKey(roundsParams),
        })
      },
    },
  })

  const courseCorrectMutation = useCourseCorrect({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({
          queryKey: getListRoundsQueryKey(roundsParams),
        })
      },
    },
  })

  const rollbackMutation = useRollback({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({
          queryKey: getListRoundsQueryKey(roundsParams),
        })
      },
    },
  })

  // ── Callbacks ──────────────────────────────────────────────────────────────
  function handleAnswer(
    questionId: string,
    answerIndex: number | null,
    answerText: string | null,
  ) {
    const round = apiRounds.find((r) =>
      r.content.questions.some((q) => q.id === questionId),
    )
    if (!round) return
    const selectedOption =
      answerIndex != null ? round.content.questions.find((q) => q.id === questionId)?.options[answerIndex] : null
    submitAnswerMutation.mutate({
      id: round.id,
      data: {
        question_id: questionId,
        selected_answer_index: answerIndex ?? undefined,
        answer_text: answerText ?? selectedOption ?? '',
      },
    })
  }

  function handleCourseCorrect(text: string) {
    const currentStage = isTaskView
      ? (apiRounds[apiRounds.length - 1]?.stage ?? 'task-qa')
      : (apiRounds[apiRounds.length - 1]?.stage ?? 'grooming')

    courseCorrectMutation.mutate({
      data: {
        text,
        stage: currentStage,
        story_id: storyId,
        task_id: taskId ?? undefined,
      },
    })
  }

  function handleRollback(roundId: string) {
    rollbackMutation.mutate({ id: roundId })
  }

  // ── Close logic ────────────────────────────────────────────────────────────
  function handleCloseAttempt() {
    if (hasPendingQuestions(apiRounds)) {
      setConfirmOpen(true)
    } else {
      void navigate({ to: '/projects/$projectId', params: { projectId } })
    }
  }

  function handleLeave() {
    setConfirmOpen(false)
    void navigate({ to: '/projects/$projectId', params: { projectId } })
  }

  // ── Task click (navigate from story → task) ────────────────────────────────
  function handleTaskClick(clickedTaskId: string) {
    void navigate({
      to: '/projects/$projectId/stories/$storyId/tasks/$taskId',
      params: { projectId, storyId, taskId: clickedTaskId },
    })
  }

  // ── Story click in breadcrumb (task → story) ───────────────────────────────
  function handleStoryClick() {
    void navigate({
      to: '/projects/$projectId/stories/$storyId',
      params: { projectId, storyId },
    })
  }

  // ── Breadcrumb ─────────────────────────────────────────────────────────────
  const storyTitle = story?.title ?? storyId
  const breadcrumbSegments = isTaskView
    ? [
        { label: projectId, onClick: handleCloseAttempt },
        { label: storyTitle, onClick: handleStoryClick },
        { label: taskId ?? '' },
      ]
    : [
        { label: projectId, onClick: handleCloseAttempt },
        { label: storyTitle },
      ]

  // ── Render ─────────────────────────────────────────────────────────────────
  if (!storyId) return null

  return (
    <>
      <ModalShell
        breadcrumb={<AppBreadcrumb segments={breadcrumbSegments} />}
        onCloseAttempt={handleCloseAttempt}
      >
        {isTaskView ? (
          <TaskViewContent
            taskId={taskId}
            storyId={storyId}
            rounds={apiRounds}
            onAnswer={handleAnswer}
            onRollback={handleRollback}
            onCourseCorrect={handleCourseCorrect}
          />
        ) : story ? (
          <StoryViewContent
            story={story}
            rounds={apiRounds}
            onAnswer={handleAnswer}
            onRollback={handleRollback}
            onCourseCorrect={handleCourseCorrect}
            onTaskClick={handleTaskClick}
          />
        ) : (
          <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
            Loading story…
          </div>
        )}
      </ModalShell>

      <ConfirmationDialog
        open={confirmOpen}
        reason="pending_questions"
        onStay={() => setConfirmOpen(false)}
        onLeave={handleLeave}
      />
    </>
  )
}
