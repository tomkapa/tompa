import { create } from 'zustand'

interface SSEStore {
  connected: boolean
  setConnected: (v: boolean) => void
}

export const useSSEStore = create<SSEStore>((set) => ({
  connected: false,
  setConnected: (connected) => set({ connected }),
}))
