import { useEffect, useRef } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { useSSEStore } from '@/stores/sse-store'
import {
  getListStoriesQueryKey,
  getGetStoryQueryKey,
} from '@/api/generated/stories/stories'
import {
  getListTasksQueryKey,
  getGetTaskQueryKey,
} from '@/api/generated/tasks/tasks'
import { getListRoundsQueryKey } from '@/api/generated/qa/qa'
import { useToastStore } from '@/stores/toast-store'

const SSE_URL = '/api/v1/events/stream'

interface StoryUpdatedData {
  story_id: string
  fields: string[]
}

interface TaskUpdatedData {
  task_id: string
  story_id: string
  fields: string[]
}

interface NewQuestionData {
  story_id: string
  task_id: string | null
  round_id: string
}

interface TaskCompletedData {
  task_id: string
  story_id: string
}

interface RefinedDescriptionReadyData {
  story_id: string
  stage: string
}

interface QuestionAssignedData {
  story_id: string
  task_id: string | null
  round_id: string
  question_id: string
  assigned_to: string
  assigned_by: string
  question_text_preview: string
}

export function useSSE(
  projectId: string,
  currentUserId: string | null,
  onNavigateToStory: ((storyId: string) => void) | null,
) {
  const setConnected = useSSEStore((s) => s.setConnected)
  const setHasNotification = useSSEStore((s) => s.setHasNotification)
  const queryClient = useQueryClient()
  const esRef = useRef<EventSource | null>(null)
  // Use a ref for the navigate callback to avoid reconnecting on every render
  const navigateRef = useRef(onNavigateToStory)
  useEffect(() => {
    navigateRef.current = onNavigateToStory
  }, [onNavigateToStory])

  useEffect(() => {
    if (!projectId) return

    const es = new EventSource(SSE_URL, { withCredentials: true })
    esRef.current = es

    es.onopen = () => setConnected(true)
    es.onerror = () => setConnected(false)

    es.addEventListener('StoryUpdated', (e: MessageEvent) => {
      const d: StoryUpdatedData = JSON.parse(e.data as string)
      void queryClient.invalidateQueries({ queryKey: getGetStoryQueryKey(d.story_id) })
      void queryClient.invalidateQueries({
        queryKey: getListStoriesQueryKey({ project_id: projectId }),
      })
    })

    es.addEventListener('TaskUpdated', (e: MessageEvent) => {
      const d: TaskUpdatedData = JSON.parse(e.data as string)
      void queryClient.invalidateQueries({ queryKey: getGetStoryQueryKey(d.story_id) })
      void queryClient.invalidateQueries({
        queryKey: getListTasksQueryKey({ story_id: d.story_id }),
      })
      void queryClient.invalidateQueries({ queryKey: getGetTaskQueryKey(d.task_id) })
      if (d.fields.includes('state')) {
        setHasNotification(true)
      }
    })

    es.addEventListener('NewQuestion', (e: MessageEvent) => {
      const d: NewQuestionData = JSON.parse(e.data as string)
      void queryClient.invalidateQueries({
        queryKey: getListRoundsQueryKey({ story_id: d.story_id }),
      })
      setHasNotification(true)
    })

    es.addEventListener('RefinedDescriptionReady', (e: MessageEvent) => {
      const d: RefinedDescriptionReadyData = JSON.parse(e.data as string)
      void queryClient.invalidateQueries({ queryKey: getGetStoryQueryKey(d.story_id) })
      setHasNotification(true)
    })

    es.addEventListener('TaskCompleted', (e: MessageEvent) => {
      const d: TaskCompletedData = JSON.parse(e.data as string)
      void queryClient.invalidateQueries({ queryKey: getGetStoryQueryKey(d.story_id) })
      void queryClient.invalidateQueries({
        queryKey: getListTasksQueryKey({ story_id: d.story_id }),
      })
      void queryClient.invalidateQueries({
        queryKey: getListStoriesQueryKey({ project_id: projectId }),
      })
    })

    es.addEventListener('QuestionAssigned', (e: MessageEvent) => {
      const d: QuestionAssignedData = JSON.parse(e.data as string)
      void queryClient.invalidateQueries({
        queryKey: getListRoundsQueryKey({ story_id: d.story_id }),
      })
      if (d.assigned_to === currentUserId && d.assigned_by !== d.assigned_to) {
        useToastStore.getState().addToast({
          variant: 'info',
          title: 'Question assigned to you',
          description: d.question_text_preview.slice(0, 100),
          action: navigateRef.current
            ? { label: 'View', onClick: () => navigateRef.current?.(d.story_id) }
            : undefined,
        })
        setHasNotification(true)
      }
    })

    return () => {
      es.close()
      esRef.current = null
      setConnected(false)
    }
  }, [projectId, currentUserId, queryClient, setConnected, setHasNotification])
}
