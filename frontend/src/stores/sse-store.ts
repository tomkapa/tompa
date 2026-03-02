import { create } from 'zustand'

interface SSEStore {
  connected: boolean
  hasNotification: boolean
  setConnected: (v: boolean) => void
  setHasNotification: (v: boolean) => void
}

export const useSSEStore = create<SSEStore>((set) => ({
  connected: false,
  hasNotification: false,
  setConnected: (connected) => set({ connected }),
  setHasNotification: (hasNotification) => set({ hasNotification }),
}))
