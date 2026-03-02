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

export function useSSE(projectId: string) {
  const setConnected = useSSEStore((s) => s.setConnected)
  const setHasNotification = useSSEStore((s) => s.setHasNotification)
  const queryClient = useQueryClient()
  const esRef = useRef<EventSource | null>(null)

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

    return () => {
      es.close()
      esRef.current = null
      setConnected(false)
    }
  }, [projectId, queryClient, setConnected, setHasNotification])
}
