import * as React from 'react'
import { X, Loader2 } from 'lucide-react'
import { IconButton } from '@/components/ui/icon-button'
import { useParams, useNavigate } from '@tanstack/react-router'
import { useQueryClient } from '@tanstack/react-query'
import { cn } from '@/lib/utils'
import { useExitAnimation } from '@/hooks/use-exit-animation'
import { AppBreadcrumb } from '@/components/ui/app-breadcrumb'
import { TabSwitcher } from '@/components/ui/tab-switcher'
import { ConfirmationDialog } from '@/components/ui/confirmation-dialog'
import { StoryOverview } from '@/features/stories/story-overview'
import { TaskOverview } from '@/features/tasks/task-overview'
import { QaThread } from '@/features/qa/qa-thread'
import { DecisionTrail } from '@/features/decisions/decision-trail'
import type { Decision, DecisionStage } from '@/features/decisions/decision-trail'
import type { AppliedPattern, QaRound } from '@/features/qa/types'
import { useGetStory, useApproveDescription, useUpdateStory, getGetStoryQueryKey } from '@/api/generated/stories/stories'
import { useGetTask, useMarkDone, getGetTaskQueryKey } from '@/api/generated/tasks/tasks'
import {
  useListRounds,
  useSubmitAnswer,
  useCourseCorrect,

  getListRoundsQueryKey,
} from '@/api/generated/qa/qa'
import type {
  QaRoundResponse,
  QaQuestion as ApiQaQuestion,
  StoryResponse,
} from '@/api/generated/tompaAPI.schemas'
import { useToastStore } from '@/stores/toast-store'
import { useAuth } from '@/hooks/use-auth'
import { putAssignee } from '@/features/qa/use-question-assignment'

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
  const appliedPatterns: AppliedPattern[] = (r.applied_patterns ?? []).map((p) => ({
    id: p.id,
    domain: p.domain,
    pattern: p.pattern,
    confidence: p.confidence,
    override_count: p.override_count,
  }))
  return {
    id: r.id,
    roundNumber: r.round_number,
    status: r.status === 'active' ? 'active' : 'superseded',
    appliedPatternCount: r.applied_pattern_count,
    appliedPatterns,
    questions: r.content.questions.map((q: ApiQaQuestion) => {
      const raw = q as unknown as Record<string, unknown>
      return {
        id: q.id,
        domain: q.domain,
        text: q.text,
        rationale: (raw.rationale as string) ?? '',
        options: q.options.map((o: unknown) =>
          typeof o === 'string'
            ? { label: o, pros: '', cons: '' }
            : (o as { label: string; pros: string; cons: string })
        ),
        recommendedIndex: (raw.recommended_option_index as number) ?? 0,
        answeredIndex: q.selected_answer_index ?? undefined,
        answeredText: q.selected_answer_text ?? undefined,
        assignedTo: (raw.assigned_to as string | null | undefined) ?? undefined,
      }
    }),
  }
}

function roundsToDecisions(rounds: QaRoundResponse[]): Decision[] {
  const decisions: Decision[] = []
  for (const round of rounds) {
    // Carry the round's applied patterns to every decision made in that round.
    const influencedByPatterns = (round.applied_patterns ?? []).map((p) => ({
      id: p.id,
      domain: p.domain,
      pattern: p.pattern,
    }))
    for (const q of round.content.questions) {
      if (q.selected_answer_index != null || q.selected_answer_text) {
        const selectedOpt = q.options[q.selected_answer_index ?? 0] as unknown
        const answerText =
          q.selected_answer_text ??
          (typeof selectedOpt === 'string' ? selectedOpt : (selectedOpt as { label: string } | undefined)?.label) ??
          ''
        decisions.push({
          id: q.id,
          domain: q.domain,
          questionText: q.text,
          answerText,
          superseded: false,
          stage: toDecisionStage(round.stage),
          answeredBy: (q as unknown as Record<string, unknown>).answered_by as string | undefined,
          influencedByPatterns: influencedByPatterns.length > 0 ? influencedByPatterns : undefined,
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
  dataState?: 'open' | 'closed'
}

function ModalShell({ breadcrumb, onCloseAttempt, children, className, dataState = 'open' }: ModalShellProps) {
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
      data-state={dataState}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 animate-in fade-in-0 data-[state=closed]:animate-out data-[state=closed]:fade-out-0"
      aria-modal
      role="dialog"
    >
      <div
        data-state={dataState}
        className={cn(
          'flex flex-col overflow-hidden bg-card',
          'h-full w-full',
          'md:h-[90vh] md:w-[90vw] md:rounded-2xl md:shadow-[0_16px_48px_rgba(0,0,0,0.2)]',
          'animate-in fade-in-0 zoom-in-[0.98]',
          'data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-[0.98]',
          className
        )}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex shrink-0 items-center justify-between border-b border-border px-4 py-3 md:px-6 md:py-4">
          {breadcrumb}
          <IconButton
            type="button"
            variant="ghost"
            aria-label="Close"
            className="h-8 w-8 text-muted-foreground"
            onClick={onCloseAttempt}
          >
            <X className="h-4 w-4" />
          </IconButton>
        </div>

        {/* Content area */}
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

const MOBILE_TABS = [
  { id: 'overview', label: 'Overview' },
  { id: 'qa', label: 'Q&A' },
  { id: 'decisions', label: 'Decisions' },
]

interface RightPanelProps {
  rounds: QaRound[]
  decisions: Decision[]
  level: 'story' | 'task'
  storyId: string
  taskId?: string
  currentStage: string
  onAnswer: (questionId: string, answerIndex: number | null, answerText: string | null) => void

  onCourseCorrect: (text: string) => void
}

function RightPanel({
  rounds,
  decisions,
  level,
  storyId,
  onAnswer,

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
            storyId={storyId}
            onAnswer={onAnswer}

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
  onCourseCorrect: (text: string) => void
  onTaskClick: (taskId: string) => void
  onApproveDescription?: () => void
  onDescriptionSave?: (description: string) => void
  isSavingDescription?: boolean
}

function StoryViewContent({
  story,
  rounds,
  onAnswer,
  onCourseCorrect,
  onTaskClick,
  onApproveDescription,
  onDescriptionSave,
  isSavingDescription,
}: Omit<StoryViewProps, 'projectId' | 'storyId'>) {
  const mappedRounds = React.useMemo(() => rounds.map(mapApiRound), [rounds])
  const decisions = React.useMemo(() => roundsToDecisions(rounds), [rounds])
  const currentStage = rounds[rounds.length - 1]?.stage ?? 'grooming'
  const [mobileTab, setMobileTab] = React.useState('overview')

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden w-full">
      {/* Mobile tab bar */}
      <div className="md:hidden shrink-0 flex justify-center border-b border-border px-4 py-2">
        <TabSwitcher tabs={MOBILE_TABS} activeId={mobileTab} onChange={setMobileTab} />
      </div>

      {/* Desktop: two-column layout */}
      <div className="hidden md:flex min-h-0 flex-1 overflow-hidden">
        <div className="w-[40%] shrink-0 overflow-y-auto border-r border-border p-4">
          <StoryOverview
            story={story}
            tasks={story.tasks}
            onTaskClick={onTaskClick}
            onApproveDescription={onApproveDescription}
            onDescriptionSave={onDescriptionSave}
            isSavingDescription={isSavingDescription}
            className="h-full"
          />
        </div>
        <RightPanel
          rounds={mappedRounds}
          decisions={decisions}
          level="story"
          storyId={story.id}
          currentStage={currentStage}
          onAnswer={onAnswer}

          onCourseCorrect={onCourseCorrect}
        />
      </div>

      {/* Mobile: single panel based on active tab */}
      <div className="flex md:hidden min-h-0 flex-1 overflow-hidden">
        {mobileTab === 'overview' && (
          <div className="flex-1 overflow-y-auto p-4">
            <StoryOverview
              story={story}
              tasks={story.tasks}
              onTaskClick={onTaskClick}
              onApproveDescription={onApproveDescription}
              onDescriptionSave={onDescriptionSave}
              isSavingDescription={isSavingDescription}
            />
          </div>
        )}
        {mobileTab === 'qa' && (
          <div className="flex-1 overflow-hidden p-4">
            <QaThread
              rounds={mappedRounds}
              storyId={story.id}
              onAnswer={onAnswer}

              onCourseCorrect={onCourseCorrect}
            />
          </div>
        )}
        {mobileTab === 'decisions' && (
          <div className="flex-1 overflow-y-auto p-4">
            <DecisionTrail decisions={decisions} level="story" />
          </div>
        )}
      </div>
    </div>
  )
}

// ── Task View (U29) ──────────────────────────────────────────────────────────

interface TaskViewContentProps {
  taskId: string
  storyId: string
  rounds: QaRoundResponse[]
  onAnswer: (questionId: string, answerIndex: number | null, answerText: string | null) => void

  onCourseCorrect: (text: string) => void
}

function TaskViewContent({
  taskId,
  rounds,
  onAnswer,

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
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to mark task as done' })
      },
    },
  })

  const mappedRounds = React.useMemo(() => rounds.map(mapApiRound), [rounds])
  const decisions = React.useMemo(() => roundsToDecisions(rounds), [rounds])
  const currentStage = rounds[rounds.length - 1]?.stage ?? 'task-qa'
  const [mobileTab, setMobileTab] = React.useState('overview')

  if (!task) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-2 text-sm text-muted-foreground">
        <Loader2 className="h-5 w-5 animate-spin" />
        Loading task…
      </div>
    )
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden w-full">
      {/* Mobile tab bar */}
      <div className="md:hidden shrink-0 flex justify-center border-b border-border px-4 py-2">
        <TabSwitcher tabs={MOBILE_TABS} activeId={mobileTab} onChange={setMobileTab} />
      </div>

      {/* Desktop: two-column layout */}
      <div className="hidden md:flex min-h-0 flex-1 overflow-hidden">
        <div className="w-[40%] shrink-0 overflow-y-auto border-r border-border p-4">
          <TaskOverview
            task={task}
            onMarkDone={() => markDoneMutation.mutate({ id: taskId })}
            markDoneLoading={markDoneMutation.isPending}
            className="h-full"
          />
        </div>
        <RightPanel
          rounds={mappedRounds}
          decisions={decisions}
          level="task"
          storyId={task.story_id}
          taskId={taskId}
          currentStage={currentStage}
          onAnswer={onAnswer}

          onCourseCorrect={onCourseCorrect}
        />
      </div>

      {/* Mobile: single panel based on active tab */}
      <div className="flex md:hidden min-h-0 flex-1 overflow-hidden">
        {mobileTab === 'overview' && (
          <div className="flex-1 overflow-y-auto p-4">
            <TaskOverview
              task={task}
              onMarkDone={() => markDoneMutation.mutate({ id: taskId })}
              markDoneLoading={markDoneMutation.isPending}
            />
          </div>
        )}
        {mobileTab === 'qa' && (
          <div className="flex-1 overflow-hidden p-4">
            <QaThread
              rounds={mappedRounds}
              storyId={task.story_id}
              onAnswer={onAnswer}

              onCourseCorrect={onCourseCorrect}
            />
          </div>
        )}
        {mobileTab === 'decisions' && (
          <div className="flex-1 overflow-y-auto p-4">
            <DecisionTrail decisions={decisions} level="task" />
          </div>
        )}
      </div>
    </div>
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
 *   /projects/:projectSlug/stories/:storyId          → Story View
 *   /projects/:projectSlug/stories/:storyId/tasks/:taskId → Task View
 */
export function StoryModal() {
  const allParams = useParams({ strict: false }) as Record<string, string | undefined>
  const projectSlug = allParams.projectSlug ?? ''
  const storyId = allParams.storyId ?? ''
  const taskId = allParams.taskId
  const isTaskView = !!taskId

  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { user } = useAuth()

  const [modalOpen, setModalOpen] = React.useState(true)
  const { visible: modalVisible, dataState: modalDataState } = useExitAnimation(modalOpen, 150)
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
  const updateDescriptionMutation = useUpdateStory({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: getGetStoryQueryKey(storyId) })
      },
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to save description' })
        console.error('[StoryModal]', { storyId, stage: 'save_description' }, 'update failed')
      },
    },
  })

  const approveDescriptionMutation = useApproveDescription({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: getGetStoryQueryKey(storyId) })
      },
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to approve description' })
      },
    },
  })

  const submitAnswerMutation = useSubmitAnswer({
    mutation: {
      onSuccess: (_, variables) => {
        void queryClient.invalidateQueries({
          queryKey: getListRoundsQueryKey(roundsParams),
        })
        if (user) {
          putAssignee(variables.id, variables.data.question_id, user.user_id).catch((err) => {
            console.warn('[StoryModal] auto-assign after answer failed', { roundId: variables.id, questionId: variables.data.question_id }, err)
          })
        }
      },
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to submit answer' })
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
      onError: () => {
        useToastStore.getState().addToast({ variant: 'error', title: 'Failed to submit course correction' })
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
    if (!round || round.status !== 'active') return
    const selectedOption =
      answerIndex != null ? round.content.questions.find((q) => q.id === questionId)?.options[answerIndex] : null
    submitAnswerMutation.mutate({
      id: round.id,
      data: {
        question_id: questionId,
        selected_answer_index: answerIndex ?? undefined,
        answer_text: answerText ?? (typeof selectedOption === 'string' ? selectedOption : selectedOption?.label) ?? '',
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

  // ── Close logic ────────────────────────────────────────────────────────────
  const navigateToProject = React.useCallback(() => {
    setModalOpen(false)
    setTimeout(() => {
      void navigate({ to: '/projects/$projectSlug', params: { projectSlug } })
    }, 150)
  }, [navigate, projectSlug])

  function handleCloseAttempt() {
    if (hasPendingQuestions(apiRounds)) {
      setConfirmOpen(true)
    } else {
      navigateToProject()
    }
  }

  function handleLeave() {
    setConfirmOpen(false)
    navigateToProject()
  }

  // ── Task click (navigate from story → task) ────────────────────────────────
  function handleTaskClick(clickedTaskId: string) {
    void navigate({
      to: '/projects/$projectSlug/stories/$storyId/tasks/$taskId',
      params: { projectSlug, storyId, taskId: clickedTaskId },
    })
  }

  // ── Story click in breadcrumb (task → story) ───────────────────────────────
  function handleStoryClick() {
    void navigate({
      to: '/projects/$projectSlug/stories/$storyId',
      params: { projectSlug, storyId },
    })
  }

  // ── Breadcrumb ─────────────────────────────────────────────────────────────
  const storyTitle = story?.title ?? storyId
  const breadcrumbSegments = isTaskView
    ? [
        { label: projectSlug, onClick: handleCloseAttempt },
        { label: storyTitle, onClick: handleStoryClick },
        { label: taskId ?? '' },
      ]
    : [
        { label: projectSlug, onClick: handleCloseAttempt },
        { label: storyTitle },
      ]

  // ── Render ─────────────────────────────────────────────────────────────────
  if (!storyId || !modalVisible) return null

  return (
    <>
      <ModalShell
        breadcrumb={<AppBreadcrumb segments={breadcrumbSegments} />}
        onCloseAttempt={handleCloseAttempt}
        dataState={modalDataState}
      >
        {isTaskView ? (
          <TaskViewContent
            taskId={taskId}
            storyId={storyId}
            rounds={apiRounds}
            onAnswer={handleAnswer}

            onCourseCorrect={handleCourseCorrect}
          />
        ) : story ? (
          <StoryViewContent
            story={story}
            rounds={apiRounds}
            onAnswer={handleAnswer}
            onCourseCorrect={handleCourseCorrect}
            onTaskClick={handleTaskClick}
            onApproveDescription={() => approveDescriptionMutation.mutate({ id: storyId, data: {} })}
            onDescriptionSave={(description) => {
              console.info('[StoryModal]', { storyId, stage: 'save_description' })
              updateDescriptionMutation.mutate({ id: storyId, data: { description } })
            }}
            isSavingDescription={updateDescriptionMutation.isPending}
          />
        ) : (
          <div className="flex flex-1 flex-col items-center justify-center gap-2 text-sm text-muted-foreground">
            <Loader2 className="h-5 w-5 animate-spin" />
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
