import { useMutation, useQueryClient } from '@tanstack/react-query'
import { getListRoundsQueryKey } from '@/api/generated/qa/qa'

export async function putAssignee(
  roundId: string,
  questionId: string,
  memberId: string,
): Promise<unknown> {
  const res = await fetch(
    `/api/v1/qa-rounds/${roundId}/questions/${questionId}/assignee`,
    {
      method: 'PUT',
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ member_id: memberId }),
    },
  )
  if (!res.ok) throw new Error('Failed to assign question')
  return res.json()
}

async function deleteAssignee(roundId: string, questionId: string): Promise<unknown> {
  const res = await fetch(
    `/api/v1/qa-rounds/${roundId}/questions/${questionId}/assignee`,
    { method: 'DELETE', credentials: 'include' },
  )
  if (!res.ok) throw new Error('Failed to unassign question')
  return res.json()
}

export function useAssignQuestion(roundId: string, questionId: string, storyId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (memberId: string) => putAssignee(roundId, questionId, memberId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: getListRoundsQueryKey({ story_id: storyId }),
      })
    },
  })
}

export function useUnassignQuestion(roundId: string, questionId: string, storyId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => deleteAssignee(roundId, questionId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: getListRoundsQueryKey({ story_id: storyId }),
      })
    },
  })
}
