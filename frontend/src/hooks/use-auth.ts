import { useMe } from '@/api/generated/auth/auth'
import type { MeResponse } from '@/api/generated/tompaAPI.schemas'

export function useAuth() {
  const { data, isLoading } = useMe({ fetch: { credentials: 'include' } })

  const isAuthenticated = data?.status === 200
  const user: MeResponse | null = isAuthenticated ? data.data : null

  return { isAuthenticated, user, isLoading }
}
