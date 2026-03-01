// Placeholder — full SSE implementation in T23
import { useEffect } from 'react'
import { useSSEStore } from '@/stores/sse-store'

export function useSSE(_projectId: string) {
  const setConnected = useSSEStore((s) => s.setConnected)

  useEffect(() => {
    // TODO: implement SSE connection in T23
    return () => {
      setConnected(false)
    }
  }, [_projectId, setConnected])
}
