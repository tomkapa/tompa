import { create } from 'zustand'

interface UIStore {
  activeStoryTab: 'qa' | 'decisions'
  courseCorrectionDraft: Record<string, string>
  setActiveStoryTab: (tab: 'qa' | 'decisions') => void
  setDraft: (id: string, text: string) => void
  clearDraft: (id: string) => void
  clearAllDrafts: () => void
}

export const useUIStore = create<UIStore>((set) => ({
  activeStoryTab: 'qa',
  courseCorrectionDraft: {},
  setActiveStoryTab: (tab) => set({ activeStoryTab: tab }),
  setDraft: (id, text) =>
    set((state) => ({
      courseCorrectionDraft: { ...state.courseCorrectionDraft, [id]: text },
    })),
  clearDraft: (id) =>
    set((state) => {
      const { [id]: _, ...rest } = state.courseCorrectionDraft
      return { courseCorrectionDraft: rest }
    }),
  clearAllDrafts: () => set({ courseCorrectionDraft: {} }),
}))
