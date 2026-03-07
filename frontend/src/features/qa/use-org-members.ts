import { useQuery } from '@tanstack/react-query'
import type { OrgMember } from './types'

export const ORG_MEMBERS_QUERY_KEY = ['/api/v1/orgs/members'] as const

async function fetchOrgMembers(): Promise<OrgMember[]> {
  const res = await fetch('/api/v1/orgs/members', { credentials: 'include' })
  if (!res.ok) throw new Error('Failed to fetch org members')
  return res.json() as Promise<OrgMember[]>
}

export function useListOrgMembers() {
  return useQuery({
    queryKey: ORG_MEMBERS_QUERY_KEY,
    queryFn: fetchOrgMembers,
  })
}
